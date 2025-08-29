use voxa_server::PluginInstance;

fn main() -> voxa_server::Result<()> {
    let config = voxa_server::ServerConfig::default();
    let plugin = test_plugin::MyPlugin::default();
    config.start_with(vec![PluginInstance::StatiC(Box::new(plugin))])
}
