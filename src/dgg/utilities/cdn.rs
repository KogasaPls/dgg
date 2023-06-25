use crate::dgg::models::flair::Flair;
use anyhow::{Context, Result};
use cached::proc_macro::cached;
use url::Url;

#[derive(Debug)]
pub struct CdnClient {
    pub url: Url,
}

impl CdnClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub async fn get_flairs(&self) -> Result<Vec<Flair>> {
        let flairs_endpoint_url = self.url.join("flairs/flairs.json")?;
        let res = reqwest::get(flairs_endpoint_url).await?;
        let flairs = res.json::<Vec<Flair>>().await?;

        Ok(flairs)
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

#[cached(size = 1, time = 604800, time_refresh = true, result = true)]
async fn get_flairs_internal(url: Url) -> Result<Vec<Flair>, Error> {
    let res = reqwest::get(url).await.context("Failed to get flairs")?;
    let flairs = res
        .json::<Vec<Flair>>()
        .await
        .context("Failed to parse flairs")?;
    Ok(flairs)
}
