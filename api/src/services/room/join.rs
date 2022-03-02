use dal::{Room, RoomExt, Dal, User, UserBuildable};
use proto::{RoomJoinRequest, RoomJoinResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use crate::services::{Payload, TypedResponse};
use tracing::instrument;

#[instrument]
pub async fn join(data: WebData, payload: Payload<RoomJoinRequest>) -> WebResult<TypedResponse<RoomJoinResponse>> {
    if payload.user_name.len() > 64 {
        return Err(Error::BadRequest("User name may not be longer than 64 characters"));
    }

    let mut room = match Room::get_by_join_code(data.dal.clone(), &payload.room_join_code)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room does not exist"))
    };

    let user = User::create(data.dal.clone(), UserBuildable {
        name: payload.user_name.clone()
    })?;
    room.add_user(&user.uuid)?;

    Ok(TypedResponse(RoomJoinResponse {
        room_uuid: room.uuid.to_string(),
        user_uuid: user.uuid.to_string(),
    }))
}