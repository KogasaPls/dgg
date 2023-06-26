use crate::common::cache::JsonCache;
use crate::dgg::models::flair::{Flair, FlairImage};
use anyhow::{Context, Result};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

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
        let mut flairs = get_flairs_internal(flairs_endpoint_url).await?;

        for flair in flairs.iter_mut() {
            if !flair.image.is_empty() {
                let img = flair.image.get_mut(0).unwrap();
                let url = img.url.parse()?;
                let bytes = get_image_bytes(url).await?;
                img.bytes = Some(bytes);
            }
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

    pub async fn ensure_flair_image_bytes_exist(&mut self, flair: &mut Flair) -> Result<()> {
        if flair.image.is_empty() {
            return Ok(());
        }

        let img = flair.image.get(0).unwrap();
        if img.bytes.is_some() {
            return Ok(());
        }

        let url = img.url.parse()?;
        let bytes = get_image_bytes(url).await?;
        let img = flair.image.get_mut(0).unwrap();
        img.bytes = Some(bytes);
        Ok(())
    }

    pub async fn get_flair_image(&self, img: &FlairImage) -> Result<Vec<u8>> {
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

async fn get_flairs_internal(url: Url) -> Result<Vec<Flair>, Error> {
    let res = reqwest::get(url).await.context("Failed to get flairs")?;
    let flairs = res
        .json::<Vec<Flair>>()
        .await
        .context("Failed to parse flairs")?;
    Ok(flairs)
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
