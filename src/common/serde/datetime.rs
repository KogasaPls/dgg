// "2019-03-24T19:18:26Z"
pub mod ymd_hms_utc {
    pub const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        dt.format(DATETIME_FORMAT).to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Utc.datetime_from_str(&time, DATETIME_FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

pub mod ymd_hms_utc_option {
    use super::ymd_hms_utc;
    use serde::Deserialize;

    pub fn serialize<S>(
        dt: &Option<chrono::DateTime<chrono::Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match dt {
            Some(dt) => ymd_hms_utc::serialize(dt, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let time: Option<String> = Option::deserialize(deserializer)?;
        match time {
            Some(time) => Ok(Some(ymd_hms_utc::deserialize(
                serde::de::IntoDeserializer::into_deserializer(time),
            )?)),
            None => Ok(None),
        }
    }
}
