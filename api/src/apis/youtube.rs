use std::collections::HashMap;
use std::fmt::Debug;
use reqwest::Result;
use crate::apis::CLIENT;
use serde::{Serialize, Deserialize};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct YouTubeApi {
    pub token: String
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResource {
    pub id: String,
    pub snippet: VideoResourceSnippet,
    pub content_details: VideoResourceContentDetails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResourceSnippet {
    title: String,
    pub thumbnails: HashMap<String, VideoThumbnail>,
    pub channel_title: String,
}

impl VideoResourceSnippet {
    pub fn get_title(&self) -> String {
        self.title.replace("- Topic", "")
    }

    pub fn get_best_thumbnail(&self) -> String {
        if let Some(x) = self.thumbnails.get("maxres") {
            &x.url
        } else if let Some(x) = self.thumbnails.get("standard") {
            &x.url
        } else if let Some(x) = self.thumbnails.get("high") {
            &x.url
        } else if let Some(x) = self.thumbnails.get("medium") {
            &x.url
        } else if let Some(x) = self.thumbnails.get("default") {
            &x.url
        } else {
            panic!("No thumbnail found");
        }.to_string()
    }
}

#[derive(Deserialize)]
pub struct VideoThumbnail {
    pub url: String,
}

#[derive(Deserialize)]
pub struct VideoResourceContentDetails {
    pub duration: String,
}

impl VideoResourceContentDetails {
    pub fn duration(&self) -> i64 {
        let dur = iso8601_duration::Duration::parse(&self.duration).expect("Parsing duration");
        dur.to_std().as_secs() as i64
    }
}

#[derive(Serialize)]
struct GetVideoQuery {
    part: String,
    id: String,
    key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    q: String,
    part: String,
    key: String,
    max_results: u32,
}

#[derive(Deserialize)]
pub struct SearchResource {
    pub id: SearchResourceId
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResourceId {
    pub video_id: String
}

impl YouTubeApi {
    #[instrument]
    pub async fn get_video<S: AsRef<str> + Debug>(&self, video_id: S) -> Result<Option<VideoResource>> {
        #[derive(Deserialize)]
        struct Response {
            items: Vec<VideoResource>
        }

        let response: Response = CLIENT.get(" https://www.googleapis.com/youtube/v3/videos")
            .query(&GetVideoQuery {
                part: "snippet,contentDetails".into(),
                id: video_id.as_ref().to_string(),
                key: self.token.clone()
            })
            .send()
            .await?
            .json()
            .await?;
        if let Some(v) = response.items.into_iter().nth(0) {
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    #[instrument]
    pub async fn search<S: AsRef<str> + Debug>(&self, q: S) -> Result<Vec<SearchResource>> {
        #[derive(Deserialize)]
        struct Response {
            items: Vec<SearchResource>
        }

        let response: Response = CLIENT.get("https://www.googleapis.com/youtube/v3/search")
            .query(&SearchQuery {
                q: q.as_ref().to_string(),
                part: "snippet".into(),
                key: self.token.clone(),
                max_results: 5,
            })
            .send()
            .await?
            .json()
            .await?;

        Ok(response.items)
    }
}