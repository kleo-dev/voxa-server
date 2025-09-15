use std::{
    collections::HashSet,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub mod macros;
pub mod requests;
pub mod types;
pub mod utils;

pub use anyhow::Result;

use crate::{utils::client::Client, utils::plugin::DynPlugin};
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
    pub db: utils::database::Database,
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
            db: utils::database::Database::new(&config).unwrap(),
            plugins: Mutex::new(Vec::new()),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(HashSet::new()),
        })
    }

    pub fn run(self: &Arc<Self>) -> Result<()> {
        // Load plugins
        #[cfg(feature = "loader")]
        utils::loader::load_plugins(
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
        'outer: loop {
            let req = client.read()?;
            if let Some(r) = &req {
                for p in self.plugins.lock().unwrap().iter_mut() {
                    if p.on_request(r, &client, self) {
                        continue 'outer;
                    }
                }

                self.call_request(r, &client)?;
            }
        }
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
