use voxa_server::{export_plugin, logger, plugin::Plugin};

logger! {
    const LOGGER "My Plugin"
}

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&mut self) {
        LOGGER.info("MyPlugin initialized!");
    }
}

export_plugin!(MyPlugin);
