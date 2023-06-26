use crate::config::ChatAppConfig;
use crate::dgg::models::event::Event;
use crate::dgg::utilities::cdn::CdnClient;
use anyhow::{bail, Context, Result};
use futures_util::stream::FusedStream;
use futures_util::StreamExt;

use tokio::net::TcpStream;

use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub struct ChatClient {
    config: ChatAppConfig,
    cdn: CdnClient,
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl ChatClient {
    pub fn new(config: ChatAppConfig) -> Self {
        Self {
            cdn: CdnClient::new(config.get_cdn_url(), config.cache_path.clone()),
            config,
            ws: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to {}", self.config.get_websocket_url());
        let ws = self.create_websocket_stream().await?;
        self.ws = Some(ws);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(ws) = &mut self.ws {
            ws.close(None).await?;
        }
        Ok(())
    }

    pub async fn get_next_event(&mut self) -> Result<Option<Event>> {
        let msg = self.get_next_message().await?;

        match msg {
            Some(msg) => {
                let event = Event::try_from(msg.as_str())?;
                Ok(Some(event))
            }
            None => Ok(None),
        }
    }

    async fn get_next_message(&mut self) -> Result<Option<String>> {
        let ws = self.ws.as_mut().context("Not connected")?;
        if ws.is_terminated() {
            bail!("Connection is closed")
        }

        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if msg.is_text() {
                debug!("Received: {}", msg.to_text()?);
                return Ok(Some(msg.into_text()?));
            }
        }

        Ok(None)
    }

    async fn create_websocket_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let tls = Connector::NativeTls(
            native_tls::TlsConnector::builder()
                .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
                .build()
                .unwrap(),
        );

        let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
            &self.config.get_websocket_url(),
            Some(self.config.websocket_config),
            false,
            Some(tls.clone()),
        )
        .await?;

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dgg::chat::mock_server::MockChatServer;
    use std::sync::LazyLock;
    use tokio::sync::Mutex;
    use tokio::test;
    use url::Url;

    const MOCK_SERVER_ADDRESS: &str = "127.0.0.1:9002";

    static MOCK_SERVER: LazyLock<Mutex<MockChatServer>> =
        LazyLock::new(|| Mutex::new(MockChatServer::new(MOCK_SERVER_ADDRESS)));

    static MOCK_SERVER_STARTED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

    fn get_test_config() -> ChatAppConfig {
        ChatAppConfig::new(
            Url::parse(&format!("ws://{}/ws", MOCK_SERVER_ADDRESS)).unwrap(),
            Url::parse(&format!("http://{}/cdn", MOCK_SERVER_ADDRESS)).unwrap(),
            None,
        )
    }

    async fn ensure_mock_server_started() {
        if *MOCK_SERVER_STARTED.lock().await {
            return;
        }

        let mut server = match MOCK_SERVER.try_lock() {
            Ok(server) => server,
            Err(_) => {
                return;
            }
        };

        if !&server.is_started {
            tokio::spawn(async move {
                server.start().await;
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            *MOCK_SERVER_STARTED.lock().await = true;
        }
    }

    #[test]
    async fn test_connect() -> Result<()> {
        ensure_mock_server_started().await;
        let mut client = ChatClient::new(get_test_config());
        client.connect().await
    }
}
