use std::{
    net::TcpListener,
    path::{Path, PathBuf},
};

#[cfg(feature = "loader")]
pub mod loader;
pub mod logger;
pub mod macros;
pub mod plugin;
pub mod vfs;

pub use anyhow::Result;

use crate::plugin::{Plugin, PluginInstance};
pub use once_cell;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    port: u16,
}

pub struct Server {
    plugins: Vec<PluginInstance>,
    root: PathBuf,
    config: ServerConfig,
    logger: logger::Logger,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 7080 }
    }
}

impl ServerConfig {
    pub fn build(self, root: &Path) -> Server {
        Server::new_config(root, self)
    }
}

impl Server {
    pub fn new(root: &Path) -> Self {
        Self {
            plugins: Vec::new(),
            root: root.to_path_buf(),
            config: ServerConfig::default(),
            logger: logger::Logger::new("Server"),
        }
    }

    pub fn new_config(root: &Path, config: ServerConfig) -> Self {
        Self {
            plugins: Vec::new(),
            root: root.to_path_buf(),
            config,
            logger: logger::Logger::new("Server"),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Load plugins
        #[cfg(feature = "loader")]
        loader::load_plugins(&mut self.plugins, &self.root.join("./plugins"))?;

        // Initialize plugins
        for plugin in &mut self.plugins {
            plugin.init();
        }

        // Start server
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port))?;
        self.logger
            .info(format!("Server listening at 0.0.0.0:{}", self.config.port));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.logger
                        .info(format!("New connection: {}", stream.peer_addr()?));
                }
                Err(e) => {
                    self.logger.error(format!("Connection failed: {}", e));
                }
            }
        }

        Ok(())
    }

    pub fn add_plugin(&mut self, plugin: PluginInstance) {
        self.plugins.push(plugin);
    }
}
