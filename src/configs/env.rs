use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
}
