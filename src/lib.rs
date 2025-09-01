use std::{
    net::TcpListener,
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
use tungstenite::accept;

use crate::plugin::Plugin;
pub use once_cell;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    port: u16,
}

pub struct Server {
    plugins: Mutex<Vec<Box<dyn Plugin>>>,
    root: PathBuf,
    config: ServerConfig,
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
        })
    }

    pub fn new_config(root: &Path, config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            plugins: Mutex::new(Vec::new()),
            root: root.to_path_buf(),
            config,
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
            plugin.init();
        }

        // Start server
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port))?;
        Self::LOGGER.info(format!("Server listening at 0.0.0.0:{}", self.config.port));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    Self::LOGGER.info(format!("New connection: {}", stream.peer_addr()?));
                    let mut ws = accept(stream)?;

                    let msg = ws.read()?;
                    ws.send(msg)?;
                }
                Err(e) => {
                    Self::LOGGER.error(format!("Connection failed: {}", e));
                }
            }
        }

        Ok(())
    }

    pub fn add_plugin(self: &Arc<Self>, plugin: Box<dyn Plugin>) {
        self.plugins.lock().unwrap().push(plugin);
    }
}
