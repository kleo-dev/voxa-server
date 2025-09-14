use std::{
    hash::{Hash, Hasher},
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::anyhow;
use serde::Serialize;

use crate::types::{ClientMessage, WsMessage};

pub mod handshake {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as Base64;
    use sha1::{Digest, Sha1};
    use std::io::{Read, Write};
    use std::net::TcpStream;

    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    pub fn handle_websocket_handshake(stream: &mut TcpStream) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        let size = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..size]);

        let key_line = request
            .lines()
            .find(|line| line.to_lowercase().starts_with("sec-websocket-key"))
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing Sec-WebSocket-Key")
            })?;

        let key = key_line.splitn(2, ':').nth(1).unwrap().trim();

        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WS_GUID.as_bytes());
        let hash = hasher.finalize();

        let accept_key = Base64.encode(hash);

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );

        stream.write_all(response.as_bytes())?;
        stream.flush()?;

        Ok(())
    }
}

pub struct Client(TcpStream);

impl Client {
    pub fn new(mut stream: TcpStream) -> crate::Result<Self> {
        handshake::handle_websocket_handshake(&mut stream)?;
        Ok(Client(stream))
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client(self.0.try_clone().expect("failed to clone TcpStream"))
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.0.peer_addr().unwrap() == other.0.peer_addr().unwrap()
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.peer_addr().unwrap().hash(state);
    }
}

impl Client {
    /// Read a full WebSocket message, handling fragmentation (FIN)
    pub fn read(&self) -> crate::Result<Option<WsMessage<ClientMessage>>> {
        let mut stream = &self.0;
        let mut message_payload = Vec::new();
        let mut final_frame = false;

        while !final_frame {
            let mut header = [0u8; 2];
            if stream.read_exact(&mut header).is_err() {
                return Ok(None); // connection closed
            }

            let fin = header[0] & 0x80 != 0;
            let opcode = header[0] & 0x0F;
            let masked = header[1] & 0x80 != 0;
            let mut payload_len = (header[1] & 0x7F) as u64;

            // Extended payload lengths
            if payload_len == 126 {
                let mut ext_len = [0u8; 2];
                stream.read_exact(&mut ext_len)?;
                payload_len = u16::from_be_bytes(ext_len) as u64;
            } else if payload_len == 127 {
                let mut ext_len = [0u8; 8];
                stream.read_exact(&mut ext_len)?;
                payload_len = u64::from_be_bytes(ext_len);
            }

            // Mask key (client → server)
            let mut mask = [0u8; 4];
            if masked {
                stream.read_exact(&mut mask)?;
            }

            // Read payload
            let mut payload = vec![0u8; payload_len as usize];
            stream.read_exact(&mut payload)?;

            if masked {
                for i in 0..payload.len() {
                    payload[i] ^= mask[i % 4];
                }
            }

            match opcode {
                0x0 | 0x1 | 0x2 => {
                    // Continuation / Text / Binary
                    message_payload.extend(payload);
                }
                0x8 => return Ok(None), // Close
                0x9 => continue,        // Ping → ignore
                0xA => continue,        // Pong → ignore
                _ => return Err(anyhow!("Unsupported WebSocket opcode: {}", opcode).into()),
            }

            final_frame = fin;
        }

        // Try parsing JSON into ClientMessage
        let message = match String::from_utf8(message_payload.clone()) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(msg) => WsMessage::Message(msg),
                Err(_) => WsMessage::String(text),
            },
            Err(_) => WsMessage::Binary(message_payload),
        };

        Ok(Some(message))
    }

    /// Send a JSON-serializable object as a WebSocket text frame
    pub fn send<T: Serialize>(&self, m: T) -> crate::Result<()> {
        let payload = serde_json::to_string(&m)?;
        let payload_bytes = payload.as_bytes();

        let mut stream = self.0.try_clone()?;
        let mut header = Vec::new();
        header.push(0x81); // FIN=1, opcode=0x1 (text)

        let len = payload_bytes.len();
        if len < 126 {
            header.push(len as u8);
        } else if len <= 65535 {
            header.push(126);
            header.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            header.push(127);
            header.extend_from_slice(&(len as u64).to_be_bytes());
        }

        stream.write_all(&header)?;
        stream.write_all(payload_bytes)?;
        stream.flush()?;

        Ok(())
    }
}
