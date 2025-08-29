use voxa_server::{export_plugin, plugin::Plugin};

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&mut self) {
        println!("MyPlugin initialized!");
    }
}

export_plugin!(MyPlugin);
