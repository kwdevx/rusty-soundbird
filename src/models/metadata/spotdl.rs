use std::error::Error;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Song {
    pub name: String,
    pub artists: Vec<String>,
    pub artist: String,
    pub genres: Vec<String>,
    pub disc_number: i32,
    pub disc_count: i32,
    pub album_name: String,
    pub album_artist: String,
    pub album_type: String,
    pub duration: i32,
    pub year: i32,
    pub date: String,
    pub track_number: i32,
    pub tracks_count: i32,
    pub song_id: String,
    pub explicit: bool,
    pub publisher: String,
    pub url: String,
    pub isrc: String,
    pub cover_url: String,
    pub copyright_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    pub popularity: i32,
    pub album_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_length: Option<i32>,
    pub artist_id: String,
}

impl Song {
    // Read songs from a JSON file
    pub async fn from_file(path: &str) -> Result<Vec<Song>, Box<dyn Error + Send + Sync>> {
        let file = File::open(path).await;
        match file {
            Ok(mut file) => {
                let mut contents = vec![];
                file.read_to_end(&mut contents).await?;

                let songs: Vec<Song> = serde_json::from_slice(&contents)?;
                Ok(songs)
            }
            Err(e) => {
                println!("{}", format!("error: {}", e));
                println!("Error reading file");
                Err(e.into())
            }
        }
    }
}
