use actix_web::web;
use dal::{Dal, Room, RoomExt};
use dal::uuid::Uuid;
use proto::TrackListResponse;
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use crate::services::payload::Payload;

pub async fn list(data: WebData, path: web::Path<Uuid>) -> WebResult<Payload<TrackListResponse>> {
    let room = match Room::get(data.dal.clone(), path.into_inner())? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room could not be found"))
    };

    let tracks = room.list_tracks()?.into_iter()
        .map(|x| proto::Track {
            track_uuid: x.uuid.to_string(),
            track_name: x.name,
            artist_name: x.artist,
            track_duration: x.duration,
            track_position: x.position,
            thumbnail_url: x.thumbnail_url,
        })
        .collect::<Vec<_>>();

    Ok(Payload(TrackListResponse {
        tracks
    }))
}