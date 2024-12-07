use poise::serenity_prelude::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use songbird::input::{AudioStream, AudioStreamError, AuxMetadata, Compose, HttpRequest, Input};
use std::{error::Error, io::ErrorKind};
use symphonia_core::io::MediaSource;
use tokio::process::Command;

use crate::input::metadata::spotdl::Output;

const SPOTIFY_DL_COMMAND: &str = "spotdl";

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
    pub client_id: String,
    pub client_secret: String,
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
    pub fn new_spotdl_like(
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
        let spotdl_args: Vec<String> = match &self.credentials {
            Some(credentials) => vec![
                "url".to_string(),
                query_str.to_string(),
                "--client-id".to_string(),
                credentials.client_id.clone(),
                "--client-secret".to_string(),
                credentials.client_secret.clone(),
            ],
            None => vec!["url".to_string(), query_str.to_string()],
        };

        let output = Command::new(self.program)
            .args(spotdl_args)
            .output()
            .await
            .map_err(|e| {
                AudioStreamError::Fail(if e.kind() == ErrorKind::NotFound {
                    format!("could not find executable '{}' on path", self.program).into()
                } else {
                    Box::new(e)
                })
            })?;

        if !output.status.success() {
            return Err(AudioStreamError::Fail(
                format!(
                    "{} failed with non-zero status code: {}",
                    self.program,
                    std::str::from_utf8(&output.stderr[..]).unwrap_or("<no error message>")
                )
                .into(),
            ));
        }

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
            .inspect(|s| {
                println!("query spotdl search result after split: {}", s);
            })
            .last()
            .inspect(|s| {
                println!("query spotdl search result after skip: {}", s);
            })
            .into_iter()
            .collect::<String>();

        // println!("query spotdl search result: {}", url);

        let out = Output {
            artist: None,
            album: None,
            channel: None,
            duration: None,
            filesize: None,
            http_headers: None,
            release_date: None,
            thumbnail: None,
            title: None,
            track: None,
            upload_date: None,
            uploader: None,
            url: url.clone(),
            webpage_url: Some(url.clone()),
        };

        let meta = out.as_aux_metadata();

        self.metadata = Some(meta);

        Ok(vec![out])
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
//     use reqwest::Client;
//
//     use super::*;
//     use crate::constants::test_data::*;
//     use crate::input::input_tests::*;
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_track_plays() {
//         track_plays_mixed(|| SpotifyDl::new(Client::new(), YTDL_TARGET.into())).await;
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_page_with_playlist_plays() {
//         track_plays_passthrough(|| SpotifyDl::new(Client::new(), YTDL_PLAYLIST_TARGET.into()))
//             .await;
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_forward_seek_correct() {
//         forward_seek_correct(|| SpotifyDl::new(Client::new(), YTDL_TARGET.into())).await;
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_backward_seek_correct() {
//         backward_seek_correct(|| SpotifyDl::new(Client::new(), YTDL_TARGET.into())).await;
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn fake_exe_errors() {
//         let mut ytdl = SpotifyDl::new_spotdl_like("yt-dlq", Client::new(), YTDL_TARGET.into());
//
//         assert!(ytdl.aux_metadata().await.is_err());
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_search_plays() {
//         let mut ytdl = SpotifyDl::new_search(Client::new(), "cloudkicker 94 days".into());
//         let res = ytdl.search(Some(1)).await;
//
//         let res = res.unwrap();
//         assert_eq!(res.len(), 1);
//
//         track_plays_passthrough(move || ytdl).await;
//     }
//
//     #[tokio::test]
//     #[ntest::timeout(20_000)]
//     async fn ytdl_search_3() {
//         let mut ytdl = SpotifyDl::new_search(Client::new(), "test".into());
//         let res = ytdl.search(Some(3)).await;
//
//         assert_eq!(res.unwrap().len(), 3);
//     }
// }
