use serde::{Deserialize, Serialize};
use tungstenite::Bytes;

/// Messages sent *from the client* (user’s app) to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a message to a channel
    SendMessage {
        channel_id: String,
        contents: String,
    },

    /// Edit a message (if allowed)
    EditMessage {
        channel_id: String,
        message_id: String,
        new_contents: String,
    },

    /// Delete a message (if allowed)
    DeleteMessage {
        channel_id: String,
        message_id: String,
    },
}

/// Messages sent *from the server* to the client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Successful authentication
    Authenticated { user_id: String },

    /// Error responses
    Error { message: String },

    /// A new message in a channel
    MessageCreate(data::Message),

    /// A message was edited
    MessageUpdate(data::Message),

    /// A message was deleted
    MessageDelete {
        channel_id: String,
        message_id: String,
    },

    /// Presence updates
    PresenceUpdate { user_id: String, status: String },

    /// Typing indicator
    Typing { user_id: String, channel_id: String },
}

/// WebSocket wrapper
#[derive(Debug, Clone)]
pub enum WsMessage<T: Serialize + for<'de> Deserialize<'de>> {
    FromClient(T),
    Binary(Bytes),
    String(String),
}

/// Shared data structures
pub mod data {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub id: String,
        pub channel_id: u8,
        pub from: String,
        pub contents: String,
        pub timestamp: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel {
        pub id: usize,
        pub name: String,
        pub kind: ChannelKind,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ChannelKind {
        Text,
        Voice,
    }
}
