use std::sync::Arc;

use crate::{Server, types, utils::client::Client};

crate::logger!(LOGGER "Message Manager");

pub fn send(
    server: &Arc<Server>,
    client: &Client,
    channel_id: &str,
    contents: &str,
) -> crate::Result<()> {
    LOGGER.info(format!("SendMessage to {channel_id}: {contents}"));

    if contents.is_empty() {
        server.wrap_err(
            &client,
            client.send(types::data::ResponseError::InvalidRequest(format!(
                "Invalid message: empty message"
            ))),
        )?;
        return Ok(());
    }

    let msg = server.wrap_err(
        &client,
        server.db.messages_db.insert(
            &channel_id,
            "idk",
            &contents,
            chrono::Utc::now().timestamp(),
        ),
    )?;

    for c in server.clients.lock().unwrap().iter() {
        server.wrap_err(&c, c.send(types::ServerMessage::MessageCreate(msg.clone())))?;
    }

    Ok(())
}

pub fn edit(
    _server: &Arc<Server>,
    _client: &Client,
    channel_id: &str,
    message_id: &str,
    new_contents: &str,
) -> crate::Result<()> {
    LOGGER.info(format!(
        "EditMessage {message_id} in {channel_id}: {new_contents}"
    ));
    Ok(())
}

pub fn delete(
    _server: &Arc<Server>,
    _client: &Client,
    channel_id: &str,
    message_id: &str,
) -> crate::Result<()> {
    LOGGER.info(format!("DeleteMessage {message_id} in {channel_id}"));
    Ok(())
}
