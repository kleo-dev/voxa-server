use std::sync::Arc;

use crate::{
    Server,
    types::{FromClient, WsMessage},
};

pub type DynPlugin = Box<dyn Plugin + Send + Sync>;

pub trait Plugin {
    fn init(&mut self, server: &Arc<Server>);
    #[allow(unused_variables)]
    fn on_request(&mut self, msg: &WsMessage<FromClient>, server: &Arc<Server>) -> bool {
        false
    }
}
