use std::sync::Arc;

use crate::Server;

pub type DynPlugin = Box<dyn Plugin + Send + Sync>;

pub trait Plugin {
    fn init(&mut self, server: &Arc<Server>);
}
