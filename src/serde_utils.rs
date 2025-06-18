// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod human_duration {
    use humantime::format_duration;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format_duration(*duration).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}

pub mod int_key_map {
    use serde::de::{self, MapAccess, Visitor};
    use serde::ser::SerializeMap;
    use serde::{Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::fmt;

    pub fn serialize<S, V>(map: &HashMap<i32, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: Serialize,
    {
        let mut keys: Vec<_> = map.keys().cloned().collect();
        keys.sort();

        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for key in keys {
            if let Some(value) = map.get(&key) {
                ser_map.serialize_entry(&key.to_string(), value)?;
            }
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<i32, i32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MapVisitor;

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = HashMap<i32, i32>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with string keys that can be parsed as integers")
            }

            fn visit_map<M>(self, mut access: M) -> Result<HashMap<i32, i32>, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut map = HashMap::new();
                while let Some((key, value)) = access.next_entry::<String, i32>()? {
                    let parsed_key = key
                        .parse::<i32>()
                        .map_err(|_| de::Error::custom(format!("Invalid integer key: {}", key)))?;
                    map.insert(parsed_key, value);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor)
    }
}
