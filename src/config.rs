use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatAppConfig {
    cdn_url_str: String,
    websocket_url_str: String,
    pub cache_path: Option<PathBuf>,
    #[serde(skip)]
    pub cdn_url: Option<Url>,
    #[serde(skip)]
    pub websocket_url: Option<Url>,
    #[serde(skip)]
    pub websocket_config: WebSocketConfig,
}

impl Default for ChatAppConfig {
    fn default() -> Self {
        ChatAppConfig::load()
    }
}

impl ChatAppConfig {
    pub fn new(cdn_url: Url, websocket_url: Url, cache_path: Option<PathBuf>) -> Self {
        Self {
            cdn_url_str: cdn_url.to_string(),
            websocket_url_str: websocket_url.to_string(),
            cdn_url: Some(cdn_url),
            cache_path,
            websocket_url: Some(websocket_url),
            websocket_config: WebSocketConfig::default(),
        }
    }

    pub fn load() -> Self {
        let config = Config::builder()
            .add_source(config::Environment::default())
            .add_source(
                config::File::with_name("config")
                    .required(true)
                    .format(config::FileFormat::Toml),
            )
            .build()
            .expect("Failed to load config");

        ChatAppConfig::try_from(config).expect("Failed to load config")
    }

    pub fn get_cdn_url(&self) -> Url {
        if let Some(cdn_url) = &self.cdn_url {
            cdn_url.clone()
        } else {
            self.cdn_url_str.parse().unwrap()
        }
    }

    pub fn get_websocket_url(&self) -> Url {
        if let Some(websocket_url) = &self.websocket_url {
            websocket_url.clone()
        } else {
            self.websocket_url_str.parse().unwrap()
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

        let cache_path = config
            .get_string("app.cache_path")
            .ok()
            .map(PathBuf::from)
            .map(|mut path| {
                if path.is_relative() {
                    let cache_dir_path = dirs::cache_dir().expect("Failed to get cache dir");
                    path = cache_dir_path.join("dgg-chat").join(path);
                }
                path
            });

        Ok(ChatAppConfig::new(cdn_url, websocket_url, cache_path))
    }
}
