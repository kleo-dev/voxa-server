use std::sync::Arc;
use voxa_server::{Server, export_plugin, logger, plugin::Plugin};

logger! {
    const LOGGER "My Plugin"
}

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&mut self, _server: &Arc<Server>) {
        LOGGER.info("MyPlugin initialized!");
    }
}

export_plugin!(Box::new(MyPlugin));
