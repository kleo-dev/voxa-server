use voxa_server::{PluginApi, export_plugin};

#[derive(Default)]
pub struct MyPlugin;

impl PluginApi for MyPlugin {
    fn init(&mut self) {
        println!("MyPlugin initialized!");
    }
}

export_plugin!(MyPlugin);
