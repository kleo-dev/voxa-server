use std::sync::Arc;

use tungstenite::Message;

use crate::Server;

pub type DynPlugin = Box<dyn Plugin + Send + Sync>;

pub trait Plugin {
    fn init(&mut self, server: &Arc<Server>);
    #[allow(unused_variables)]
    fn on_request(&mut self, msg: &Message, server: &Arc<Server>) -> bool {
        false
    }
}
