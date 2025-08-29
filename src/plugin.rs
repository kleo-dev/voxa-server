#[cfg(feature = "loader")]
use wasmtime::{Instance, Store};
#[cfg(feature = "loader")]
use wasmtime_wasi::preview1::WasiP1Ctx;

pub trait Plugin {
    fn init(&mut self);
}

pub enum PluginInstance {
    #[cfg(feature = "loader")]
    Wasm(Instance, Store<WasiP1Ctx>),
    Static(Box<dyn Plugin>),
}

impl Plugin for PluginInstance {
    fn init(&mut self) {
        match self {
            #[cfg(feature = "loader")]
            PluginInstance::Wasm(instance, store) => {
                let init = instance
                    .get_typed_func::<(), ()>(&mut *store, "init")
                    .unwrap();
                init.call(store, ()).unwrap();
            }
            PluginInstance::Static(plugin) => plugin.init(),
        }
    }
}
