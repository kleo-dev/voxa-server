pub mod message;

use std::sync::Arc;

use crate::{
    Server,
    types::{ClientMessage, WsMessage},
    utils::client::Client,
};

impl Server {
    pub fn call_request(
        self: &Arc<Self>,
        req: &WsMessage<ClientMessage>,
        client: &Client,
    ) -> crate::Result<()> {
        match req {
            WsMessage::Message(req) => match req {
                ClientMessage::SendMessage {
                    channel_id,
                    contents,
                } => {
                    message::send(self, client, channel_id, contents)?;
                }

                ClientMessage::EditMessage {
                    channel_id,
                    message_id,
                    new_contents,
                } => message::edit(self, client, channel_id, message_id, new_contents)?,

                ClientMessage::DeleteMessage {
                    channel_id,
                    message_id,
                } => message::delete(self, client, channel_id, message_id)?,
            },

            WsMessage::Binary(b) => {
                Self::LOGGER.info(format!("Binary message: {b:?}"));
            }

            WsMessage::String(s) => {
                Self::LOGGER.info(format!("String message: {s}"));
            }
        }

        Ok(())
    }
}
