//! Websocket data types

use std::net::TcpStream;

use tungstenite::{client, stream::MaybeTlsStream, Message, WebSocket};

#[derive(Default)]
pub struct Connection {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl Connection {
    pub fn connect(&mut self, url: &str) -> tungstenite::Result<()> {
        let (mut socket, _) = client::connect(url)?;

        // NOTE: `MaybeTlsStream` only has `Plain` variant unless we enable their TLS feature
        // TODO: Enable security feature
        if let MaybeTlsStream::Plain(s) = socket.get_mut() {
            // Do not block or our game would be choppy animations
            s.set_nonblocking(true).unwrap();
        }

        self.socket = Some(socket);

        Ok(())
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        let socket = self.socket.as_mut()?;

        // TODO: do not discard fallible message?
        let msg = socket.read_message().ok()?;

        if let Message::Binary(buf) = msg {
            Some(buf)
        } else {
            None
        }
    }
}
