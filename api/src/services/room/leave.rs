use dal::{Dal, RemoveStatus, Room, RoomExt, uuid::Uuid};
use proto::{RoomLeaveRequest, RoomLeaveResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use crate::services::payload::{Payload, TypedResponse};
use tracing::instrument;

#[instrument]
pub async fn leave(data: WebData, payload: Payload<RoomLeaveRequest>) -> WebResult<TypedResponse<RoomLeaveResponse>> {
    let mut room = match Room::get(data.dal.clone(), Uuid::parse_str(&payload.room_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room does not exist"))
    };

    match room.remove_user(&Uuid::parse_str(&payload.user_uuid)?)? {
        RemoveStatus::LastMember => {
            room.delete()?;
            Ok(TypedResponse(RoomLeaveResponse {
                deleted: true,
                new_owner: None,
            }))
        },
        RemoveStatus::Ok { new_owner} =>  {
            Ok(TypedResponse(RoomLeaveResponse {
                deleted: false,
                new_owner: Some(new_owner.to_string())
            }))
        }
    }
}