use std::path::PathBuf;
use voxa_server::plugin::PluginInstance;

fn main() -> voxa_server::Result<()> {
    let root = PathBuf::from("../");
    let config = voxa_server::ServerConfig::default();
    let plugin = test_plugin::MyPlugin::default();
    config.start_with(vec![PluginInstance::Static(Box::new(plugin))], &root)
}
