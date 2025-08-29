use std::path::Path;

use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

use crate::{PluginInstance, vfs};

pub fn load_plugin(wasm_path: &Path) -> anyhow::Result<PluginInstance> {
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

pub fn load_plugins(arr: &mut Vec<PluginInstance>, path: &Path) -> crate::Result<()> {
    vfs::dir(path)?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                println!("Loading plugin: {:?}", path);
                match load_plugin(&path) {
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

    Ok(())
}
