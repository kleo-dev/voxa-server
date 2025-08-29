#[cfg(feature = "loader")]
use wasmtime::{Instance, Store};
#[cfg(feature = "loader")]
use wasmtime_wasi::preview1::WasiP1Ctx;

#[cfg(feature = "loader")]
pub mod loader;
pub mod macros;

pub use anyhow::Result;

pub trait PluginApi {
    fn init(&mut self);
}

pub struct ServerConfig {}

pub enum PluginInstance {
    #[cfg(feature = "loader")]
    Wasm(Instance, Store<WasiP1Ctx>),
    StatiC(Box<dyn PluginApi>),
}

impl PluginApi for PluginInstance {
    fn init(&mut self) {
        match self {
            #[cfg(feature = "loader")]
            PluginInstance::Wasm(instance, store) => {
                let init = instance
                    .get_typed_func::<(), ()>(&mut *store, "init")
                    .unwrap();
                init.call(store, ()).unwrap();
            }
            PluginInstance::StatiC(plugin) => plugin.init(),
        }
    }
}

impl ServerConfig {
    pub fn start(&self) -> Result<()> {
        let mut plugins: Vec<PluginInstance> = Vec::new();
        #[cfg(feature = "loader")]
        loader::load_plugins(&mut plugins, std::path::Path::new("./plugins"));

        self.start_with(plugins)
    }

    pub fn start_with(&self, mut plugins: Vec<PluginInstance>) -> Result<()> {
        for plugin in &mut plugins {
            plugin.init();
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {}
    }
}
