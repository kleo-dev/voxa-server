use std::sync::Arc;
use voxa_server::{Server, export_plugin, logger, utils::plugin::Plugin};

logger! {
    const LOGGER "My Plugin"
}

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn init(&mut self, _server: &Arc<Server>) {
        LOGGER.info("MyPlugin initialized!");
    }

    fn on_request(
        &mut self,
        msg: &voxa_server::types::message::WsMessage<voxa_server::types::message::ClientMessage>,
        client: &voxa_server::utils::client::Client,
        _server: &Arc<Server>,
    ) -> bool {
        LOGGER.info(&format!("Received message: {:?}", msg));
        match msg {
            voxa_server::types::message::WsMessage::Message(
                voxa_server::types::message::ClientMessage::SendMessage { contents, .. },
            ) => {
                if contents == "ping" {
                    LOGGER.info("Pong!");
                    // client
                    //     .send(voxa_server::types::message::ServerMessage::TempMessage {
                    //         message: "pong".to_string(),
                    //     })
                    //     .unwrap();
                    client
                        .send(voxa_server::types::message::ServerMessage::TempMessage {
                            message: "pong".to_string(),
                        })
                        .unwrap();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

export_plugin!(Box::new(MyPlugin));
