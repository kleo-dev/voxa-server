use std::path::PathBuf;

use voxa_server::{ServerConfig, plugin::PluginInstance, vfs};

fn main() -> voxa_server::Result<()> {
    let root = PathBuf::from("./");
    let config: ServerConfig = vfs::read_config(&root.join("config.json"))?;
    let mut server = config.build(&root);
    server.add_plugin(PluginInstance::Static(Box::new(
        test_plugin::MyPlugin::default(),
    )));
    server.run()?;
    Ok(())
}
