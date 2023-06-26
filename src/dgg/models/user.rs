use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub nick: String,
    pub features: Vec<String>,
    #[serde(with = "crate::common::serde::datetime::ymd_hms_utc")]
    pub created_date: DateTime<Utc>,
}
