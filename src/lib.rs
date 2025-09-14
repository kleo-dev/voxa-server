use std::{
    collections::HashSet,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub mod client;
pub mod database;
#[cfg(feature = "loader")]
pub mod loader;
pub mod logger;
pub mod macros;
pub mod plugin;
pub mod types;
pub mod vfs;

pub use anyhow::Result;
pub use tungstenite;

use crate::{
    client::Client,
    plugin::DynPlugin,
    types::{ClientMessage, WsMessage},
};
pub use once_cell;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    port: u16,
    channels: Vec<types::data::Channel>,
}

#[allow(dead_code)]
pub struct Server {
    root: PathBuf,
    config: ServerConfig,
    plugins: Mutex<Vec<DynPlugin>>,
    clients: Mutex<HashSet<Client>>,
    pub db: database::Database,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7080,
            channels: Vec::new(),
        }
    }
}

impl ServerConfig {
    pub fn build(self, root: &Path) -> Arc<Server> {
        Server::new_config(root, self)
    }
}

impl Server {
    logger!(LOGGER "Server");

    pub fn new(root: &Path) -> Arc<Self> {
        Self::new_config(root, ServerConfig::default())
    }

    pub fn new_config(root: &Path, config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            db: database::Database::new(&config).unwrap(),
            plugins: Mutex::new(Vec::new()),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(HashSet::new()),
        })
    }

    pub fn run(self: &Arc<Self>) -> Result<()> {
        // Load plugins
        #[cfg(feature = "loader")]
        loader::load_plugins(
            &mut *self.plugins.lock().unwrap(),
            &self.root.join("./plugins"),
        )?;

        // Initialize plugins
        for plugin in self.plugins.lock().unwrap().iter_mut() {
            plugin.init(self);
        }

        // Start server
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port))?;
        Self::LOGGER.info(format!("Server listening at 0.0.0.0:{}", self.config.port));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    std::thread::spawn({
                        let srv = self.clone();
                        move || match srv.handle_client(stream) {
                            Ok(_) => {}
                            Err(e) => Self::LOGGER.error(format!("Client handler failed: {e}")),
                        }
                    });
                }
                Err(e) => {
                    Self::LOGGER.error(format!("Connection failed: {e}"));
                }
            }
        }

        Ok(())
    }

    pub fn add_plugin(self: &Arc<Self>, plugin: DynPlugin) {
        self.plugins.lock().unwrap().push(plugin);
    }

    fn handle_client(self: &Arc<Self>, stream: TcpStream) -> anyhow::Result<()> {
        Self::LOGGER.info(format!("New connection: {}", stream.peer_addr()?));
        // Initialize client
        let client = Client::new(stream)?;

        // Insert to the set of all connected clients
        self.clients.lock().unwrap().insert(client.clone());

        // The main req/res loop
        loop {
            match client.read()? {
                Some(WsMessage::Message(req)) => match req {
                    ClientMessage::SendMessage {
                        channel_id,
                        contents,
                    } => {
                        Self::LOGGER.info(format!("SendMessage to {channel_id}: {contents}"));
                        let msg = self.wrap_err(
                            &client,
                            self.db.messages_db.insert(
                                &channel_id,
                                "idk",
                                &contents,
                                chrono::Utc::now().timestamp(),
                            ),
                        )?;

                        self.wrap_err(
                            &client,
                            client.send(types::ServerMessage::MessageCreate(msg)),
                        )?;
                    }

                    ClientMessage::EditMessage {
                        channel_id,
                        message_id,
                        new_contents,
                    } => {
                        Self::LOGGER.info(format!(
                            "EditMessage {message_id} in {channel_id}: {new_contents}"
                        ));
                    }

                    ClientMessage::DeleteMessage {
                        channel_id,
                        message_id,
                    } => {
                        Self::LOGGER.info(format!("DeleteMessage {message_id} in {channel_id}"));
                    }
                },

                Some(WsMessage::Binary(b)) => {
                    Self::LOGGER.info(format!("Binary message: {b:?}"));
                }

                Some(WsMessage::String(s)) => {
                    Self::LOGGER.info(format!("String message: {s}"));
                }

                None => {
                    self.clients.lock().unwrap().remove(&client);
                    break;
                }
            }
        }

        Ok(())
    }

    /// When there is a error it removes the client
    pub fn wrap_err<T, E>(
        self: &Arc<Self>,
        client: &Client,
        res: std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        if res.is_err() {
            self.clients.lock().unwrap().remove(&client);
        }

        res
    }
}
