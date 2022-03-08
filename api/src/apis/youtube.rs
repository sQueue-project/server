use std::collections::HashMap;
use std::fmt::Debug;
use reqwest::Result;
use crate::apis::CLIENT;
use serde::{Serialize, Deserialize};
use tracing::{info, instrument};

#[derive(Debug)]
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
    pub title: String,
    pub thumbnails: HashMap<String, VideoThumbnail>,
    pub channel_title: String,
}

impl VideoResourceSnippet {
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
struct Query {
    part: String,
    id: String,
    key: String,
}

impl YouTubeApi {
    #[instrument]
    pub async fn get_video<S: AsRef<str> + Debug>(&self, video_id: S) -> Result<Option<VideoResource>> {
        #[derive(Deserialize)]
        struct Response {
            items: Vec<VideoResource>
        }

        info!("Do we get here?");

        let response_text = CLIENT.get(" https://www.googleapis.com/youtube/v3/videos")
            .query(&Query {
                part: "snippet,contentDetails".into(),
                id: video_id.as_ref().to_string(),
                key: self.token.clone()
            })
            .send()
            .await?
            .text()
            .await?;

        info!("{}", response_text);

        let response: Response = serde_json::from_str(&response_text).unwrap();

        if let Some(v) = response.items.into_iter().nth(0) {
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
}