use crate::config::ChatAppConfig;
use crate::dgg::models::event::{ChatMessageData, Event, EventData};
use crate::dgg::utilities::cdn::CdnClient;
use anyhow::{anyhow, bail, Context, Result};
use futures_util::stream::FusedStream;
use futures_util::{SinkExt, TryStreamExt};

use tokio::net::TcpStream;

use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::{http, Message};
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub enum WebSocketMessage {
    Event(Event),
    Ping,
    Pong,
    Close,
}

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

    pub async fn send_message(&mut self, message: String) -> Result<()> {
        let msg = Event::ChatMessage(EventData::<ChatMessageData> {
            data: ChatMessageData { data: message },
            base: Default::default(),
        });
        let msg_str: String = msg.try_into()?;

        let ws = self.ws.as_mut().context("Not connected")?;
        debug!("Sending: {}", msg_str);
        ws.send(Message::Text(msg_str)).await?;
        Ok(())
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

    pub async fn get_next_message(&mut self) -> Result<Option<WebSocketMessage>> {
        let ws = self.ws.as_mut().context("Not connected")?;
        if ws.is_terminated() {
            bail!("Connection is closed")
        }

        match ws.try_next().await? {
            Some(msg) => match msg {
                Message::Text(msg) => {
                    let event = Event::try_from(msg.as_str())?;
                    Ok(Some(WebSocketMessage::Event(event)))
                }
                Message::Binary(b) => Err(anyhow!("I didn't expect to get one of these: {:?}", b)),
                Message::Ping(_) => {
                    trace!("Got ping, sending pong");
                    ws.send(Message::Pong(vec![])).await?;
                    Ok(Some(WebSocketMessage::Ping))
                }
                Message::Pong(_) => Ok(Some(WebSocketMessage::Pong)),
                Message::Close(_) => {
                    debug!("Got close, closing");
                    ws.close(None).await?;
                    Ok(Some(WebSocketMessage::Close))
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    async fn create_websocket_stream(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let tls = Connector::NativeTls(
            native_tls::TlsConnector::builder()
                .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
                .build()
                .unwrap(),
        );

        let origin_url = self.config.get_origin_url();
        let websocket_url = self.config.get_websocket_url();

        let request = Request::builder()
            .uri(websocket_url.as_str())
            .method("GET")
            .header("Host", websocket_url.host_str().unwrap())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .version(http::Version::HTTP_11)
            .header("Origin", origin_url.as_str())
            .header("User-Agent", "KogasaPls/dgg")
            .header("authtoken", self.config.token.as_ref().unwrap())
            .body(())?;

        let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
            request,
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
            Url::parse(&format!("https://{}/", MOCK_SERVER_ADDRESS)).unwrap(),
            Url::parse(&format!("ws://{}/ws", MOCK_SERVER_ADDRESS)).unwrap(),
            Url::parse(&format!("https://{}/cdn", MOCK_SERVER_ADDRESS)).unwrap(),
            None,
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
