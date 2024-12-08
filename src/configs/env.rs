use std::sync::Arc;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(with = "rc_string_serde")]
    pub discord_token: Arc<String>,
    #[serde(with = "rc_string_serde")]
    pub spotify_client_id: Arc<String>,
    #[serde(with = "rc_string_serde")]
    pub spotify_client_secret: Arc<String>,
}
// Module containing serialization/deserialization logic
mod rc_string_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    // Serialize just the String contents
    pub fn serialize<S>(rc_str: &Arc<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        rc_str.serialize(serializer)
    }

    // Deserialize into an Arc<String>
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Arc::new)
    }
}
