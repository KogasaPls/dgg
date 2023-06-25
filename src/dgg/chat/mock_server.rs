use futures_util::{SinkExt, StreamExt};
use log::*;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};

pub struct MockChatServer {
    pub address: String,
    pub is_started: bool,
}

impl MockChatServer {
    pub fn new(addr: &str) -> Self {
        Self {
            address: addr.to_owned(),
            is_started: false,
        }
    }

    async fn accept_connection(&self, peer: SocketAddr, stream: TcpStream) {
        if let Err(e) = self.handle_connection(peer, stream).await {
            match e {
                Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
                err => error!("Error processing connection: {:?}", err),
            }
        }
    }

    async fn handle_connection(&self, peer: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

        info!("New WebSocket connection: {}", peer);

        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if msg.is_text() || msg.is_binary() {
                ws_stream.send(msg).await?;
            }
        }

        Ok(())
    }

    pub async fn start(&mut self) {
        let listener = TcpListener::bind(&self.address)
            .await
            .expect("Can't listen");
        info!("Listening on: {}", &self.address);

        self.is_started = true;

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("connected streams should have a peer address");
            info!("Peer address: {}", peer);

            self.accept_connection(peer, stream).await;
        }
    }
}
