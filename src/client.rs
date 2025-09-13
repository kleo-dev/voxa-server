use std::{
    hash::{Hash, Hasher},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use tungstenite::{Message, Utf8Bytes, WebSocket, accept};

use crate::types::{FromClient, ToClient, WsMessage};

#[derive(Clone)]
pub struct Client(Arc<Mutex<WebSocket<TcpStream>>>);

impl Client {
    pub fn new_ws(ws: WebSocket<TcpStream>) -> Self {
        Self(Arc::new(Mutex::new(ws)))
    }

    pub fn new_tcp(ws: TcpStream) -> crate::Result<Self> {
        Ok(Self(Arc::new(Mutex::new(accept(ws)?))))
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state)
    }
}

impl Client {
    pub fn read(&self) -> crate::Result<Option<WsMessage<FromClient>>> {
        match self.0.lock().unwrap().read()? {
            Message::Text(t) => {
                let v = t.to_string();
                match serde_json::from_str(&v) {
                    Ok(f) => Ok(Some(WsMessage::FromClient(f))),
                    Err(_) => Ok(Some(WsMessage::String(v))),
                }
            }

            Message::Binary(b) => Ok(Some(WsMessage::Binary(b))),

            Message::Close(_) => Ok(None),

            m => Err(Error::msg(format!("Invalid websocket format: {m}"))),
        }
    }

    pub fn send(&self, m: ToClient) -> crate::Result<()> {
        self.0
            .lock()
            .unwrap()
            .send(Message::Text(Utf8Bytes::from(serde_json::to_string(&m)?)))
            .map_err(|e| e.into())
    }
}
