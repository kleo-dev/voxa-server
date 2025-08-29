use std::path::Path;

#[cfg(feature = "loader")]
pub mod loader;
pub mod macros;
pub mod plugin;
pub mod vfs;

pub use anyhow::Result;

use crate::plugin::{Plugin, PluginInstance};

pub struct ServerConfig {}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {}
    }
}

impl ServerConfig {
    pub fn start(&self, root: &Path) -> Result<()> {
        let mut plugins: Vec<PluginInstance> = Vec::new();
        #[cfg(feature = "loader")]
        loader::load_plugins(&mut plugins, &root.join("./plugins"))?;

        self.start_with(plugins, root)
    }

    pub fn start_with(&self, mut plugins: Vec<PluginInstance>, root: &Path) -> Result<()> {
        for plugin in &mut plugins {
            plugin.init();
        }

        Ok(())
    }
}
