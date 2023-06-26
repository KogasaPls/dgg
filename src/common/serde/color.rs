pub mod hex {
    use palette::rgb::Rgb;
    use serde::Deserialize;

    pub fn serialize<S>(color: &Rgb, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "#{:02x}{:02x}{:02x}",
            (color.red * 255.0).round() as u8,
            (color.green * 255.0).round() as u8,
            (color.blue * 255.0).round() as u8
        ))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rgb, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() == 7 && s.starts_with('#') {
            let r = u8::from_str_radix(&s[1..3], 16);
            let g = u8::from_str_radix(&s[3..5], 16);
            let b = u8::from_str_radix(&s[5..7], 16);

            match (r, g, b) {
                (Ok(r), Ok(g), Ok(b)) => Ok(Rgb::from([r, g, b]).into_format()),
                _ => Err(serde::de::Error::custom(format!("Invalid color: {}", s))),
            }
        } else {
            Err(serde::de::Error::custom(format!("Invalid color: {}", s)))
        }
    }

    #[cfg(test)]
    mod tests {
        use palette::rgb::Rgb;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct SerializableColor {
            #[serde(with = "super")]
            pub color: Rgb,
        }

        test_serde!(
            SerializableColor,
            (
                SerializableColor {
                    color: Rgb::from([0_u8, 0_u8, 0_u8]).into_format()
                },
                "{\"color\":\"#000000\"}"
            )
        );
    }
}

pub mod hex_option {
    use palette::rgb::Rgb;
    use serde::Deserialize;

    pub fn serialize<S>(color: &Option<Rgb>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match color {
            Some(color) => super::hex::serialize(color, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Rgb>, D::Error>
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
