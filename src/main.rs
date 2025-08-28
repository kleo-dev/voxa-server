use wasmtime::*;

fn load_plugin(wasm_path: &str) -> anyhow::Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());

    let module = Module::from_file(&engine, wasm_path)?;
    let instance = linker.instantiate(&mut store, &module)?;

    let init = instance.get_typed_func::<(), ()>(&mut store, "init")?;
    init.call(&mut store, ())?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    load_plugin("plugins/test_plugin.wasm")?;
    Ok(())
}
