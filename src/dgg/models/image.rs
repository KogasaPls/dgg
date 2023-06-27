use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Image {
    pub url: String,
    pub name: String,
    pub mime: String,
    pub height: u16,
    pub width: u16,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
}
