use serde::{Deserialize, Serialize};
use tungstenite::Bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum FromClient {
    SendMessage(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum FromServer {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum ToClient {
    Message(data::Message),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum ToServer {}

/// WebSocket for client messages
#[derive(Debug, Clone)]
pub enum WsMessage<T: Serialize + for<'de> Deserialize<'de>> {
    FromClient(T),
    Binary(Bytes),
    String(String),
}

pub mod data {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub from: String,
        pub contents: String,
    }
}
