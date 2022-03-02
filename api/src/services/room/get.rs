use actix_web::web;
use crate::appdata::WebData;
use dal::{Room, Uuid, Dal, User};
use proto::RoomInfoResponse;
use crate::error::{Error, WebResult};
use crate::services::TypedResponse;

pub async fn get(data: WebData, path: web::Path<Uuid>) -> WebResult<TypedResponse<RoomInfoResponse>> {
    let room = match Room::get(data.dal.clone(), path.into_inner())? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room does not exist"))
    };

    let owner = match User::get(data.dal.clone(), room.owner.clone())? {
        Some(x) => x,
        None => return Err(Error::Conflict("The room's owner does not exist".to_string()))
    };

    Ok(TypedResponse(RoomInfoResponse {
        room_uuid: room.uuid.to_string(),
        owner_uuid: room.owner.to_string(),
        join_code: room.join_code.to_string(),
        room_name: room.name,
        owner_name: owner.name
    }))
}