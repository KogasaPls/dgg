use crate::common::cache::JsonCache;
use crate::dgg::models::flair::Flair;
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::dgg::models::emote::Emote;
use crate::dgg::models::image::Image;
use std::path::PathBuf;
use url::Url;

#[derive(Debug)]
pub struct CdnClient {
    pub url: Url,
    cache: Option<JsonCache>,
}

impl CdnClient {
    pub fn new(url: Url, cache_path: Option<PathBuf>) -> Self {
        Self {
            url,
            cache: cache_path.map(JsonCache::new),
        }
    }

    pub async fn get_emotes(&mut self) -> Result<HashMap<String, Emote>> {
        if let Some(cache) = self.cache.as_mut() {
            let emotes = cache.get("emotes".to_string())?;
            if let Some(emotes) = emotes {
                info!("Using cached emotes");
                return Ok(emotes);
            }
        }

        info!("Not using cached emotes");
        let emotes_endpoint_url = self.url.join("emotes/emotes.json")?;
        let res = reqwest::get(emotes_endpoint_url)
            .await
            .context("Failed to get emotes")?;

        let mut emotes = res
            .json::<Vec<Emote>>()
            .await
            .context("Failed to parse emotes")?;

        for emote in emotes.iter_mut() {
            ensure_image_bytes_exist(&mut emote.image[0]).await?;
        }

        let emotes_map = emotes
            .into_iter()
            .map(|e| (e.prefix.clone(), e))
            .collect::<HashMap<String, Emote>>();

        if let Some(cache) = self.cache.as_mut() {
            cache.set("emotes".to_string(), emotes_map.clone())?;
            Ok(emotes_map)
        } else {
            Ok(emotes_map)
        }
    }

    pub async fn get_flairs(&mut self) -> Result<HashMap<String, Flair>> {
        if let Some(cache) = self.cache.as_mut() {
            let flairs = cache.get("flairs".to_string())?;
            if let Some(flairs) = flairs {
                info!("Using cached flairs");
                return Ok(flairs);
            }
        }

        info!("Not using cached flairs");
        let flairs_endpoint_url = self.url.join("flairs/flairs.json")?;
        let res = reqwest::get(flairs_endpoint_url)
            .await
            .context("Failed to get flairs")?;
        let mut flairs = res
            .json::<Vec<Flair>>()
            .await
            .context("Failed to parse flairs")?;

        for flair in flairs.iter_mut() {
            ensure_image_bytes_exist(&mut flair.image[0]).await?;
        }

        let flairs_map = flairs
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect::<HashMap<String, Flair>>();

        if let Some(cache) = self.cache.as_mut() {
            cache.set("flairs".to_string(), flairs_map.clone())?;
            Ok(flairs_map)
        } else {
            Ok(flairs_map)
        }
    }

    pub async fn get_image(&self, img: &Image) -> Result<Vec<u8>> {
        let url = img.url.parse()?;
        let bytes = get_image_bytes(url).await?;
        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

async fn get_image_bytes(url: Url) -> Result<Vec<u8>, Error> {
    let res = reqwest::get(url).await.context("Failed to get image")?;
    let bytes = res
        .bytes()
        .await
        .context("Failed to get image bytes")?
        .to_vec();
    Ok(bytes)
}

async fn ensure_image_bytes_exist(image: &mut Image) -> Result<()> {
    if image.bytes.as_ref().is_some_and(|b| !b.is_empty()) {
        return Ok(());
    }

    let url = image.url.parse()?;
    let bytes = get_image_bytes(url).await?;
    image.bytes = Some(bytes);
    Ok(())
}
