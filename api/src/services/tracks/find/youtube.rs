use proto::{TrackFindYouTubeRequest, TrackFindYouTubeResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use actix_multiresponse::Payload;
use dal::{User, Room, Dal, Pretrack, PretrackBuildable, PretrackSourcePlatform};
use dal::uuid::Uuid;
use crate::apis::youtube::YouTubeApi;
use tracing::instrument;

#[instrument]
pub async fn youtube(data: WebData, payload: Payload<TrackFindYouTubeRequest>) -> WebResult<Payload<TrackFindYouTubeResponse>> {
    match Room::get(data.dal.clone(), Uuid::parse_str(&payload.room_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested Room does not exist"))
    };

    match User::get(data.dal.clone(), Uuid::parse_str(&payload.user_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested User does not exist"))
    };

    let api = YouTubeApi { token: data.config.google_api_key.clone() };
    let video = match api.get_video(&payload.youtube_id).await? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested YouTube video does not exist")),
    };

    let pretrack = Pretrack::create(data.dal.clone(), PretrackBuildable {
        duration: video.content_details.duration(),
        thumbnail_url: video.snippet.get_best_thumbnail(),
        name: video.snippet.get_title(),
        artist: video.snippet.channel_title,
        platform: PretrackSourcePlatform::YouTube(payload.youtube_id.clone())
    })?;

    Ok(Payload(TrackFindYouTubeResponse {
        pretrack_uuid: pretrack.uuid.to_string()
    }))
}
