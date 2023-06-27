use crate::dgg::models::image::Image;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Emote {
    pub prefix: String,
    pub creator: Option<String>,
    pub twitch: bool,
    pub theme: u8,
    pub minimum_sub_tier: u8,
    pub image: Vec<Image>,
}
