use std::{
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[cfg(feature = "loader")]
pub mod loader;
pub mod logger;
pub mod macros;
pub mod plugin;
pub mod vfs;

pub use anyhow::Result;
pub use tungstenite;
use tungstenite::{WebSocket, accept};

use crate::plugin::DynPlugin;
pub use once_cell;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    port: u16,
}

pub struct Server {
    root: PathBuf,
    config: ServerConfig,
    plugins: Mutex<Vec<DynPlugin>>,
    clients: Mutex<Vec<Arc<Mutex<WebSocket<TcpStream>>>>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 7080 }
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
        Arc::new(Self {
            plugins: Mutex::new(Vec::new()),
            root: root.to_path_buf(),
            config: ServerConfig::default(),
            clients: Mutex::new(Vec::new()),
        })
    }

    pub fn new_config(root: &Path, config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            plugins: Mutex::new(Vec::new()),
            root: root.to_path_buf(),
            config,
            clients: Mutex::new(Vec::new()),
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
        let ws = Arc::new(Mutex::new(accept(stream)?));

        self.clients.lock().unwrap().push(ws.clone());

        loop {
            let req = ws.lock().unwrap().read()?;

            for plugin in self.plugins.lock().unwrap().iter_mut() {
                plugin.on_request(&req, self);
            }

            if req.is_close() {
                let mut clients = self.clients.lock().unwrap();

                if let Some(i) = clients.iter().position(|v| Arc::ptr_eq(v, &ws)) {
                    clients.remove(i);
                }
            }

            for c in self.clients.lock().unwrap().iter_mut() {
                self.wrap_err(&ws, c.lock().unwrap().send(req.clone()))?;
            }
        }

        Ok(())
    }

    /// When there is a error it removes the client
    fn wrap_err<T, E>(
        self: &Arc<Self>,
        ws: &Arc<Mutex<WebSocket<TcpStream>>>,
        res: std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        if res.is_err() {
            let mut clients = self.clients.lock().unwrap();
            if let Some(i) = clients.iter().position(|v| Arc::ptr_eq(v, &ws)) {
                clients.remove(i);
            }
        }

        res
    }
}
