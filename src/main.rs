use voxa_server::PluginApi;

fn main() -> anyhow::Result<()> {
    let mut plugins = Vec::new();
    voxa_server::loader::load_plugins(&mut plugins, std::path::Path::new("./plugins"));

    for plugin in &mut plugins {
        plugin.init();
    }

    Ok(())
}
