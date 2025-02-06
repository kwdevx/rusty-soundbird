#[allow(dead_code)]
use crate::input::metadata::spotdl::Output;
use crate::models::metadata::spotdl::Song;
use anyhow::Result;
use core::option::Option;
use poise::serenity_prelude::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use songbird::input::{AudioStream, AudioStreamError, AuxMetadata, Compose, HttpRequest, Input};
use std::{error::Error, io::ErrorKind, sync::Arc};
use symphonia_core::io::MediaSource;
use tokio::process::Command;

const SPOTIFY_DL_COMMAND: &str = "spotdl";

// for getting download url from spotdl
const SPOTIFY_DL_OPTION_URL: &str = "url";

// for getting metadata from spotdl
const SPOTIFY_DL_OPTION_SAVE: &str = "save";
const SPOTIFY_DL_OPTION_SAVE_SAVE_FILE_FLAG: &str = "--save-file";

const SPOTIFY_DL_FILE_NAME: &str = "temp.spotdl";

const SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_ID_FLAG: &str = "--client-id";
const SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_SECRET_FLAG: &str = "--client-secret";

#[derive(Clone, Debug)]
enum QueryType {
    UrlOrSearch(String),
}

/// A lazily instantiated call to download a file, finding its URL via youtube-dl.
///
/// By default, this uses yt-dlp and is backed by an [`HttpRequest`]. This handler
/// attempts to find the best audio-only source (typically `WebM`, enabling low-cost
/// Opus frame passthrough).
///
/// [`HttpRequest`]: super::HttpRequest
#[derive(Clone, Debug)]
pub struct SpotifyDl {
    program: &'static str,
    client: Client,
    metadata: Option<AuxMetadata>,
    query: QueryType,
    credentials: Option<SpotifyCredential>,
}

#[derive(Debug, Clone)]
pub struct SpotifyCredential {
    pub client_id: Arc<String>,
    pub client_secret: Arc<String>,
}

impl SpotifyDl {
    /// Creates a lazy request to select an audio stream from `url`, using "yt-dlp".
    ///
    /// This requires a reqwest client: ideally, one should be created and shared between
    /// all requests.
    #[must_use]
    pub fn new(client: Client, url: String, credentials: Option<SpotifyCredential>) -> Self {
        Self::new_spotdl_like(SPOTIFY_DL_COMMAND, client, url, credentials)
    }

    /// Creates a lazy request to select an audio stream from `url` as in [`new`], using `program`.
    ///
    /// [`new`]: Self::new
    #[must_use]
    fn new_spotdl_like(
        program: &'static str,
        client: Client,
        url: String,
        credentials: Option<SpotifyCredential>,
    ) -> Self {
        Self {
            program,
            client,
            metadata: None,
            query: QueryType::UrlOrSearch(url),
            credentials,
        }
    }

    async fn query(&mut self) -> Result<Vec<Output>, AudioStreamError> {
        let query_str = match &self.query {
            QueryType::UrlOrSearch(url) => url,
        };
        let url = self.process_url_command(query_str).await;

        let meta = self.process_save_command(query_str).await;

        match (url, meta) {
            (Ok(url), Ok(meta)) => {
                println!("Both query and meta are Ok");
                println!("query result: {}", url);
                println!("meta result: {:?}", meta);
                let out = Output {
                    artist: Option::from(meta.artist),
                    album: Option::from(meta.album_name),
                    channel: None,
                    duration: Option::from(meta.duration as f64),
                    filesize: None,
                    http_headers: None,
                    release_date: Option::from(meta.date),
                    thumbnail: Option::from(meta.cover_url),
                    title: Option::from(meta.name),
                    track: Option::from(meta.track_number.to_string()),
                    upload_date: None,
                    uploader: None,
                    url: url.clone(),
                    webpage_url: Some(url.clone()),
                };

                self.metadata = Some(out.as_aux_metadata());

                Ok(vec![out])
            }
            (Err(e), Ok(_)) => {
                println!("query error: {}", e);
                Err(AudioStreamError::Fail(Box::new(e)))
            }
            (Ok(_), Err(e)) => {
                println!("meta error: {}", e);
                Err(AudioStreamError::Fail(Box::new(e)))
            }
            (Err(e1), Err(e2)) => {
                println!("Both query and meta are Err");
                println!("query error: {}", e1);
                println!("meta error: {}", e2);
                Err(AudioStreamError::Fail(Box::new(e1)))
            }
        }
    }

    async fn process_url_command(&self, query_str: &String) -> Result<String, AudioStreamError> {
        let spotdl_url_args: Vec<&str> = match &self.credentials {
            Some(credentials) => vec![
                SPOTIFY_DL_OPTION_URL,
                query_str,
                SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_ID_FLAG,
                credentials.client_id.as_ref(),
                SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_SECRET_FLAG,
                credentials.client_secret.as_ref(),
            ],
            None => vec![SPOTIFY_DL_OPTION_URL, query_str],
        };
        let url_output = Command::new(self.program)
            .args(spotdl_url_args)
            .output()
            .await
            .map_err(|e| {
                AudioStreamError::Fail(if e.kind() == ErrorKind::NotFound {
                    format!("could not find executable '{}' on path", self.program).into()
                } else {
                    Box::new(e)
                })
            });

        match url_output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(AudioStreamError::Fail(
                        format!(
                            "{} failed with non-zero status code: {}",
                            self.program,
                            std::str::from_utf8(&output.stderr[..]).unwrap_or("<no error message>")
                        )
                        .into(),
                    ));
                };

                // NOTE: must be split_mut for spotdl result and skip the first two lines which are
                // [Processing query: <query>] and [url: <url>]
                // spotdl result is not json format, e.g
                // # spotdl url "suger for the pill"
                // Processing query: suger for the pill
                // https://rr2---sn-cxaaj5o5q5-tt1ek.googlevideo.com/videoplayback?expire=173362572...
                let url = String::from_utf8(output.stdout)
                    .map_err(|e| AudioStreamError::Fail(Box::new(e)))?
                    .trim()
                    .split('\n')
                    .last()
                    .into_iter()
                    .collect::<String>();

                Ok(url)
            }
            Err(e) => Err(AudioStreamError::Fail(Box::new(e))),
        }
    }

    async fn process_save_command(&self, query_str: &String) -> Result<Song, AudioStreamError> {
        let spotdl_save_args: Vec<&str> = match &self.credentials {
            Some(credentials) => vec![
                SPOTIFY_DL_OPTION_SAVE,
                query_str,
                SPOTIFY_DL_OPTION_SAVE_SAVE_FILE_FLAG,
                SPOTIFY_DL_FILE_NAME,
                SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_ID_FLAG,
                credentials.client_id.as_ref(),
                SPOTIFY_DL_OPTION_SPOTIFY_CLIENT_SECRET_FLAG,
                credentials.client_secret.as_ref(),
            ],
            None => vec![
                SPOTIFY_DL_OPTION_SAVE,
                query_str,
                SPOTIFY_DL_OPTION_SAVE_SAVE_FILE_FLAG,
                SPOTIFY_DL_FILE_NAME,
            ],
        };

        match Command::new(self.program).args(spotdl_save_args).spawn() {
            Ok(mut child) => match child.wait().await {
                    Ok(status) => {
                        if !status.success() {
                            return Err(AudioStreamError::Fail(
                                format!("{} failed with non-zero status code", self.program).into(),
                            ));
                        }
                        // Process completed successfully, handle the result here
                    match Song::from_file(SPOTIFY_DL_FILE_NAME).await {
                            Ok(songs) => {
                            if songs.is_empty() {
                                    Err(AudioStreamError::Fail("No song found in the file".into()))
                            } else {
                                Ok(songs[0].clone())
                                }
                            }
                            Err(e) => Err(AudioStreamError::Fail(e)),
                        }
                    }
                Err(e) => Err(AudioStreamError::Fail(Box::new(e))),
            },
            Err(e) => Err(AudioStreamError::Fail(Box::new(e))),
        }
    }
}

impl From<SpotifyDl> for Input {
    fn from(val: SpotifyDl) -> Self {
        Input::Lazy(Box::new(val))
    }
}

#[async_trait]
impl Compose for SpotifyDl {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        Err(AudioStreamError::Unsupported)
    }

    async fn create_async(
        &mut self,
    ) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        // panic safety: `query` should have ensured > 0 results if `Ok`
        let mut results = self.query().await?;

        match results.first() {
            Some(ele) => {
                println!("create_async result: {}", ele.url);
            }
            None => {
                println!("create_async no result");
            }
        }

        let result = results.swap_remove(0);

        let mut headers = HeaderMap::default();

        if let Some(map) = result.http_headers {
            headers.extend(map.iter().filter_map(|(k, v)| {
                Some((
                    HeaderName::from_bytes(k.as_bytes()).ok()?,
                    HeaderValue::from_str(v).ok()?,
                ))
            }));
        }

        let mut req = HttpRequest {
            client: self.client.clone(),
            request: result.url,
            headers,
            content_length: result.filesize,
        };

        req.create_async().await
    }

    fn should_create_async(&self) -> bool {
        true
    }

    async fn aux_metadata(&mut self) -> Result<AuxMetadata, AudioStreamError> {
        if let Some(meta) = self.metadata.as_ref() {
            return Ok(meta.clone());
        }

        self.query().await?;

        self.metadata.clone().ok_or_else(|| {
            let msg: Box<dyn Error + Send + Sync + 'static> =
                "Failed to instansiate any metadata... Should be unreachable.".into();
            AudioStreamError::Fail(msg)
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_track_plays() {
//         let songs = Song::from_file(SPOTIFY_DL_FILE_NAME).expect("spotdl save result is not json");
//
//         track_plays_mixed(|| SpotifyDl::new(Client::new(), YTDL_TARGET.into())).await;
//     }
//
// }
