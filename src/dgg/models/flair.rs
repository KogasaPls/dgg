use anyhow::Context;
use rgb::RGB8;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase", from = "&str")]
pub enum FlairKind {
    Moderator,
    Protected,
    Subscriber,
    TwitchSubscriber,
    Admin,
    Broadcaster,
    Vip,
    Bot,
    CommunityBot,
    SubscriberTier1,
    SubscriberTier2,
    SubscriberTier3,
    SubscriberTier4,
    SubscriberTier5,
    Micro,
    NflAndy,
    Verified,
    NewUser,
    LeagueMaster,
    Gym,
    Lawyer,
    TikTokEditor,
    YouTubeContributor,
    YouTubeEditor,
    DndKnightTierParty,
    DndKnightTierScoria,
    DndBaronTierGold,
    EmoteMaster,
    EmoteContributor,
    ShirtDesigner,
    Birthday,
    MinecraftVip,
    StarCraft2,
    CompositionWinner,
    Contributor,
    Trusted,
    Notable,
    #[serde(untagged)]
    Other(String),
}

// c.f. https://cdn.destiny.gg/flairs/flairs.json
impl From<&str> for FlairKind {
    fn from(value: &str) -> Self {
        match value.strip_prefix("flair").map(|value| {
            value
                .parse::<u64>()
                .context("Expected flair to be followed by a number")
                .unwrap()
        }) {
            Some(number) => match number {
                1 => FlairKind::SubscriberTier2,
                2 => FlairKind::Notable,
                3 => FlairKind::SubscriberTier3,
                4 => FlairKind::Trusted,
                5 => FlairKind::Contributor,
                6 => FlairKind::CompositionWinner,
                7 => FlairKind::NflAndy,
                8 => FlairKind::SubscriberTier4,
                9 => FlairKind::TwitchSubscriber,
                10 => FlairKind::StarCraft2,
                11 => FlairKind::CommunityBot,
                12 => FlairKind::Broadcaster,
                13 => FlairKind::SubscriberTier1,
                14 => FlairKind::MinecraftVip,
                15 => FlairKind::Birthday,
                16 => FlairKind::EmoteContributor,
                17 => FlairKind::Micro,
                18 => FlairKind::EmoteMaster,
                19 => FlairKind::ShirtDesigner,
                20 => FlairKind::Verified,
                21 => FlairKind::YouTubeEditor,
                22 => FlairKind::DndBaronTierGold,
                24 => FlairKind::DndKnightTierScoria,
                25 => FlairKind::YouTubeContributor,
                26 => FlairKind::DndKnightTierParty,
                27 => FlairKind::TikTokEditor,
                28 => FlairKind::Lawyer,
                29 => FlairKind::Gym,
                30 => FlairKind::LeagueMaster,
                42 => FlairKind::SubscriberTier5,
                58 => FlairKind::NewUser,
                _ => FlairKind::Other(value.to_owned()),
            },
            None => match value {
                "moderator" => FlairKind::Moderator,
                "protected" => FlairKind::Protected,
                "subscriber" => FlairKind::Subscriber,
                "admin" => FlairKind::Admin,
                "vip" => FlairKind::Vip,
                "bot" => FlairKind::Bot,
                _ => FlairKind::Other(value.to_owned()),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Flair {
    pub label: String,
    pub name: String,
    pub description: Option<String>,
    pub hidden: bool,
    pub priority: u64,
    #[serde(with = "crate::common::serde::color::hex_option")]
    pub color: Option<RGB8>,
    pub rainbow_color: bool,
    pub image: Vec<FlairImage>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct FlairImage {
    pub url: String,
    pub name: String,
    pub mime: String,
    pub height: u8,
    pub width: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::include_resource;

    #[test]
    fn test_flair_deserialization() {
        let flairs = include_resource!("flairs.json");
        let flairs: Vec<Flair> = serde_json::from_str(&flairs).unwrap();

        debug!("{:#?}", flairs);
    }
}
