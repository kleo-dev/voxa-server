use std::{
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::types::{ClientMessage, WsMessage};

pub mod handshake {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as Base64;
    use sha1::{Digest, Sha1};
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;

    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    pub fn handle_websocket_handshake(stream: &mut TcpStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        if !request_line.starts_with("GET") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid HTTP method",
            ));
        }

        let mut headers = HashMap::new();
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 || line == "\r\n" {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }

        let key = headers.get("sec-websocket-key").ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing Sec-WebSocket-Key")
        })?;

        if headers.get("upgrade").map(|v| v.to_lowercase()) != Some("websocket".to_string()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid Upgrade header",
            ));
        }

        if !headers
            .get("connection")
            .map(|v| v.to_lowercase().contains("upgrade"))
            .unwrap_or(false)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid Connection header",
            ));
        }

        // Optional: validate Sec-WebSocket-Version == 13 (most common)
        if let Some(ver) = headers.get("sec-websocket-version") {
            if ver.trim() != "13" {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported Sec-WebSocket-Version",
                ));
            }
        }

        // Compute accept key
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WS_GUID.as_bytes());
        let hash = hasher.finalize();
        let accept_key = Base64.encode(hash);

        // Note: include Sec-WebSocket-Protocol handling if you support subprotocols
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n",
            accept_key
        );

        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

pub struct Client(TcpStream, Option<String>, u64);

impl Client {
    /// Create a client with no timeouts
    pub fn new(mut stream: TcpStream) -> crate::Result<Self> {
        handshake::handle_websocket_handshake(&mut stream)?;
        Ok(Client(stream, None, rand::random()))
    }

    /// Create a client and set read/write timeouts (useful in prod)
    pub fn with_timeouts(
        mut stream: TcpStream,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> crate::Result<Self> {
        if let Some(t) = read_timeout {
            stream.set_read_timeout(Some(t))?;
        }
        if let Some(t) = write_timeout {
            stream.set_write_timeout(Some(t))?;
        }
        handshake::handle_websocket_handshake(&mut stream)?;
        Ok(Client(stream, None, rand::random()))
    }

    /// Send a close frame and flush. `code` is a WebSocket close code (e.g., 1000 normal).
    pub fn send_close(&self, code: u16, reason: &str) -> crate::Result<()> {
        let mut stream = self.0.try_clone()?;

        // control frames must be <= 125 bytes
        let mut payload = Vec::new();
        payload.extend_from_slice(&code.to_be_bytes());
        payload.extend_from_slice(reason.as_bytes());
        if payload.len() > 125 {
            return Err(anyhow!("close reason too long").into());
        }

        let mut frame = Vec::with_capacity(2 + payload.len());
        frame.push(0x88); // FIN=1, opcode=0x8 (Close)
        frame.push(payload.len() as u8); // server->client MUST NOT mask
        frame.extend_from_slice(&payload);

        stream.write_all(&frame)?;
        stream.flush()?;
        Ok(())
    }

    /// Send a pong with given payload (control frames must be <=125)
    fn send_pong(&self, payload: &[u8]) -> crate::Result<()> {
        let mut stream = self.0.try_clone()?;

        if payload.len() > 125 {
            return Err(anyhow!("pong payload too long").into());
        }
        let mut frame = Vec::with_capacity(2 + payload.len());
        frame.push(0x8A); // FIN=1, opcode=0xA (Pong)
        frame.push(payload.len() as u8);
        frame.extend_from_slice(payload);
        stream.write_all(&frame)?;
        stream.flush()?;
        Ok(())
    }

    /// Send a text/binary frame (server->client must NOT mask)
    pub fn send<T: Serialize>(&self, m: T) -> crate::Result<()> {
        let mut stream = self.0.try_clone()?;

        let payload = serde_json::to_string(&m)?;
        let payload_bytes = payload.as_bytes();
        let len = payload_bytes.len();

        let mut header = Vec::new();
        header.push(0x81); // FIN=1, opcode=0x1 (text)

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

    /// Read a full WebSocket message, handling fragmentation and control frames.
    ///
    /// Returns:
    /// - Ok(Some(WsMessage)) on an application message (text/binary)
    /// - Ok(None) if the connection should be closed (close received / read EOF)
    /// - Err on protocol or IO errors.
    pub fn read_t<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
    ) -> crate::Result<Option<WsMessage<T>>> {
        let mut stream = self.0.try_clone()?;

        let mut message_payload = Vec::new();

        loop {
            // read the 2-byte header
            let mut header = [0u8; 2];
            if let Err(e) = stream.read_exact(&mut header) {
                if e.kind() == io::ErrorKind::UnexpectedEof || e.kind() == io::ErrorKind::BrokenPipe
                {
                    return Ok(None); // treat EOF as closed
                }
                return Err(e.into());
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

            // Mask key (client→server MUST be masked)
            let mut mask = [0u8; 4];
            if masked {
                stream.read_exact(&mut mask)?;
            } else {
                let _ = self.send_close(1002, "Client frames must be masked");
                return Ok(None);
            }

            // Control frame checks
            if matches!(opcode, 0x8 | 0x9 | 0xA) {
                if payload_len > 125 {
                    let _ = self.send_close(1002, "Control frame too large");
                    return Ok(None);
                }
                if !fin {
                    let _ = self.send_close(1002, "Control frames must not be fragmented");
                    return Ok(None);
                }
            }

            // Read payload + unmask
            let mut payload = vec![0u8; payload_len as usize];
            if payload_len > 0 {
                stream.read_exact(&mut payload)?;
                for i in 0..payload.len() {
                    payload[i] ^= mask[i % 4];
                }
            }

            match opcode {
                0x0 | 0x1 | 0x2 => {
                    // Continuation / Text / Binary
                    message_payload.extend(payload);
                    if fin {
                        break; // got full message
                    } else {
                        continue; // wait for more fragments
                    }
                }
                0x8 => {
                    // Close
                    let (code, reason) = if payload.len() >= 2 {
                        let code = u16::from_be_bytes([payload[0], payload[1]]);
                        let reason = if payload.len() > 2 {
                            String::from_utf8_lossy(&payload[2..]).into_owned()
                        } else {
                            String::new()
                        };
                        (code, reason)
                    } else {
                        (1000, String::new())
                    };
                    let _ = self.send_close(code, &reason);
                    return Ok(None);
                }
                0x9 => {
                    // Ping → respond with Pong
                    let _ = self.send_pong(&payload);
                    continue;
                }
                0xA => {
                    // Pong → ignore
                    continue;
                }
                _ => {
                    let _ = self.send_close(1002, "Unsupported opcode");
                    return Ok(None);
                }
            }
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

    /// Read a full WebSocket message, handling fragmentation and control frames.
    ///
    /// Returns:
    /// - Ok(Some(WsMessage)) on an application message (text/binary)
    /// - Ok(None) if the connection should be closed (close received / read EOF)
    /// - Err on protocol or IO errors.
    pub fn read(&self) -> crate::Result<Option<WsMessage<ClientMessage>>> {
        self.read_t()
    }

    pub fn get_uuid(&self) -> crate::Result<String> {
        match &self.1 {
            Some(v) => Ok(v.clone()),
            None => Err(anyhow!("Client ({}) UUID not set", self.2).into()),
        }
    }

    pub fn set_uuid(&mut self, uuid: &str) {
        self.1 = Some(uuid.to_string())
    }

    #[deprecated]
    pub fn addr(&self) -> crate::Result<SocketAddr> {
        Ok(self.0.peer_addr().unwrap_or(self.0.local_addr()?))
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client(
            self.0.try_clone().expect("failed to clone TcpStream"),
            self.1.clone(),
            self.2.clone(),
        )
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.2 == other.2
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.2.hash(state);
    }
}
