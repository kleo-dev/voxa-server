#[cfg(feature = "loader")]
use wasmtime::{Instance, Store};
#[cfg(feature = "loader")]
use wasmtime_wasi::preview1::WasiP1Ctx;

#[cfg(feature = "loader")]
pub mod loader;

pub trait PluginApi {
    fn init(&mut self);
}

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

#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        static PLUGIN: std::sync::Mutex<Option<$plugin_type>> = std::sync::Mutex::new(None);

        #[unsafe(no_mangle)]
        pub extern "C" fn init() {
            let mut plugin = PLUGIN.lock().unwrap();
            *plugin = Some(<$plugin_type>::default());
            if let Some(plugin) = plugin.as_mut() {
                plugin.init();
            }
        }
    };
}
