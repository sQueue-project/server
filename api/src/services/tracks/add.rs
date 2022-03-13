use dal::{Dal, Pretrack, Room, RoomExt, Track, TrackBuildable, User};
use dal::uuid::Uuid;
use proto::{SsePacketEvent, TrackAddRequest, TrackAddResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use actix_multiresponse::Payload;
use tracing::instrument;

#[instrument]
pub async fn add(data: WebData, payload: Payload<TrackAddRequest>) -> WebResult<Payload<TrackAddResponse>> {
    let room = match Room::get(data.dal.clone(), Uuid::parse_str(&payload.room_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested Room does not exist"))
    };

    let user = match User::get(data.dal.clone(), Uuid::parse_str(&payload.user_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested User does not exist"))
    };

    if room.list_members()?
        .iter()
        .filter(|x| x.uuid.eq(&user.uuid))
        .collect::<Vec<_>>()
        .is_empty() {
       return Err(Error::Forbidden("User is not in Room"));
    }

    let pretrack = match Pretrack::get(data.dal.clone(), Uuid::parse_str(&payload.pretrack_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested Pretrack does not exist"))
    };

    let track = Track::create(data.dal.clone(), TrackBuildable {
        room_uuid: room.uuid,
        pretrack
    })?;

    let track_idx = room.get_queue()?.enqueue(&track, &user.uuid)?;

    let proto_track = proto::Track {
        track_uuid: track.uuid.to_string(),
        track_name: track.name,
        artist_name: track.artist,
        track_duration: track.duration,
        thumbnail_url: track.thumbnail_url,
        track_idx
    };

    let sse = data.sse_track_list.lock();
    if let Some(broadcaster) = sse.get(&room.uuid) {
        let lock = broadcaster.lock();
        lock.send(SsePacketEvent::Data, proto_track.clone())?;
    }

    Ok(Payload(TrackAddResponse {
        track: Some(proto_track)
    }))
}
