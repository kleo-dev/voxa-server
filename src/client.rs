use std::{
    hash::{Hash, Hasher},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use tungstenite::{Message, WebSocket, accept};

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
    pub fn read(&self) -> crate::Result<Message> {
        self.0.lock().unwrap().read().map_err(|e| e.into())
    }

    pub fn send(&self, m: Message) -> crate::Result<()> {
        self.0.lock().unwrap().send(m).map_err(|e| e.into())
    }
}
