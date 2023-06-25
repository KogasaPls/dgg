use anyhow::{Context, Result};
use config::Config;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

#[derive(Debug, Clone)]
pub struct ChatAppConfig {
    pub cdn_url: Url,
    pub websocket_url: Url,
    pub websocket_config: WebSocketConfig,
}

impl ChatAppConfig {
    pub fn new(cdn_url: Url, websocket_url: Url) -> Self {
        Self {
            cdn_url,
            websocket_url,
            websocket_config: WebSocketConfig::default(),
        }
    }
}

impl TryFrom<Config> for ChatAppConfig {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let mut cdn_url: Url = config
            .get_string("dgg.cdn_url")
            .context("Failed to get dgg.cdn_url")?
            .parse()?;
        cdn_url
            .set_scheme("https")
            .map_err(|_| anyhow::anyhow!("Failed to set scheme to https"))?;

        let mut websocket_url: Url = config
            .get_string("dgg.websocket_url")
            .context("Failed to get dgg.websocket_url")?
            .parse()?;
        websocket_url
            .set_scheme("wss")
            .map_err(|_| anyhow::anyhow!("Failed to set scheme to wss"))?;

        Ok(ChatAppConfig {
            cdn_url,
            websocket_url,
            websocket_config: WebSocketConfig::default(),
        })
    }
}
