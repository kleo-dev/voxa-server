use std::path::Path;

use wasmtime::*;
use wasmtime_wasi::{WasiCtxBuilder, preview1::WasiP1Ctx};

use crate::PluginApi;

pub enum PluginInstance {
    Wasm(Instance, Store<WasiP1Ctx>),
    StatiC(Box<dyn PluginApi>),
}

pub fn load_plugin(wasm_path: &str) -> anyhow::Result<PluginInstance> {
    let engine = Engine::default();
    let mut store = Store::new(
        &engine,
        WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build_p1(),
    );
    let module = Module::from_file(&engine, wasm_path)?;
    let mut linker = Linker::new(&engine);

    wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |s| s)?;

    let instance = linker.instantiate(&mut store, &module)?;
    Ok(PluginInstance::Wasm(instance, store))
}

pub fn load_plugins(arr: &mut Vec<PluginInstance>, path: &Path) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                println!("Loading plugin: {:?}", path);
                match load_plugin(path.to_str().unwrap()) {
                    Ok(plugin) => {
                        arr.push(plugin);
                    }
                    Err(e) => eprintln!("Failed to load plugin {:?}: {}", path, e),
                }
            }
        }
    } else {
        eprintln!("{:?} is not a directory", path);
    }
}

impl PluginApi for PluginInstance {
    fn init(&mut self) {
        match self {
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
