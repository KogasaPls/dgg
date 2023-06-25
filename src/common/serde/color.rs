pub mod hex {
    use rgb::RGB8;
    use serde::Deserialize;

    pub fn serialize<S>(color: &RGB8, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<RGB8, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() == 7 && s.starts_with('#') {
            let r = u8::from_str_radix(&s[1..3], 16);
            let g = u8::from_str_radix(&s[3..5], 16);
            let b = u8::from_str_radix(&s[5..7], 16);
            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Ok(RGB8 { r, g, b }),
                _ => Err(serde::de::Error::custom(format!("Invalid color: {}", s))),
            }
        } else {
            Err(serde::de::Error::custom(format!("Invalid color: {}", s)))
        }
    }

    #[cfg(test)]
    mod tests {

        use rgb::RGB8;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct SerializableColor {
            #[serde(with = "super")]
            pub color: RGB8,
        }

        test_serde!(
            SerializableColor,
            (
                SerializableColor {
                    color: RGB8::new(0, 0, 0)
                },
                "{\"color\":\"#000000\"}"
            )
        );
    }
}

pub mod hex_option {
    use rgb::RGB8;
    use serde::Deserialize;

    pub fn serialize<S>(color: &Option<RGB8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match color {
            Some(color) => super::hex::serialize(color, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<RGB8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            Some(s) => {
                if s.is_empty() {
                    return Ok(None);
                }

                super::hex::deserialize(serde::de::IntoDeserializer::into_deserializer(s)).map(Some)
            }
            None => Ok(None),
        }
    }
}
