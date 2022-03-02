use dal::{Dal, Room, RoomBuildable, RoomExt, User, UserBuildable};
use proto::{CreateRoomRequest, RoomCreatedResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use crate::services::{Payload, TypedResponse};
use tracing::instrument;

#[instrument]
pub async fn create(data: WebData, payload: Payload<CreateRoomRequest>) -> WebResult<TypedResponse<RoomCreatedResponse>> {
    if payload.user_name.len() > 64 {
        return Err(Error::BadRequest("User name may not be longer than 64 characters"));
    }

    if payload.room_name.len() > 64 {
        return Err(Error::BadRequest("Room name may not be longer than 64 characters"));
    }

    let user = User::create(data.dal.clone(), UserBuildable {
        name: payload.user_name.to_string()
    })?;

    let mut room = Room::create(data.dal.clone(), RoomBuildable {
        name: payload.room_name.to_string(),
        user_owner: user.uuid.clone()
    })?;
    room.add_user(&user.uuid)?;

    Ok(TypedResponse(RoomCreatedResponse {
        room_uuid: room.uuid.to_string(),
        owner_uuid: user.uuid.to_string(),
        join_code: room.join_code
    }))
}