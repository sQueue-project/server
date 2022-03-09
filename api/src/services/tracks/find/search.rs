use actix_web::web;
use crate::appdata::WebData;
use serde::Deserialize;
use tracing::instrument;
use proto::TrackFindSearchResponse;
use crate::apis::youtube::YouTubeApi;
use crate::error::WebResult;
use crate::services::payload::Payload;

#[derive(Deserialize, Debug)]
pub struct Query {
    q: String,
}

#[instrument]
pub async fn search(data: WebData, query: web::Query<Query>) -> WebResult<Payload<TrackFindSearchResponse>> {

    let youtube_result = tokio::spawn(async move {
        search_youtube(YouTubeApi { token: data.config.google_api_key.clone() }, &query.q).await
    });

    let youtube_result = tokio::join! {
        youtube_result
    };

    let youtube_result = youtube_result.0??;
    Ok(Payload(TrackFindSearchResponse {
        tracks: youtube_result
    }))
}

#[instrument]
async fn search_youtube(youtube_api: YouTubeApi, query: &str) -> WebResult<Vec<proto::TrackFindSearchTrack>> {
    let search = youtube_api.search(query).await?;
    let handles = search.into_iter()
        .map(|x| {
            let api = youtube_api.clone();
            tokio::spawn(async move {
                api.clone().get_video(x.id.video_id).await
            })
        })
        .collect::<Vec<_>>();
    let handled = futures::future::join_all(handles).await;
    let tracks = handled.into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|x| x.ok())
        .filter_map(|x| x)
        .map(|x| {
            proto::TrackFindSearchTrack {
                thumbnail_url: x.snippet.get_best_thumbnail(),
                name: x.snippet.get_title(),
                duration: x.content_details.duration(),
                youtube_id: Some(x.id),
                spotify_id: None,
                artist: x.snippet.channel_title,
            }
        })
        .collect::<Vec<_>>();
    Ok(tracks)
}