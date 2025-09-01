use std::sync::Arc;

use crate::Server;

pub trait Plugin {
    fn init(&mut self, server: &Arc<Server>);
}
