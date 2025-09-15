use std::sync::Arc;

use crate::{
    Server,
    types::{ClientMessage, WsMessage},
    utils::client::Client,
};

pub type DynPlugin = Box<dyn Plugin + Send + Sync>;

pub trait Plugin {
    fn init(&mut self, server: &Arc<Server>);
    #[allow(unused_variables)]
    fn on_request(
        &mut self,
        req: &WsMessage<ClientMessage>,
        client: &Client,
        server: &Arc<Server>,
    ) -> bool {
        false
    }
}
