use dal::{Dal, RemoveStatus, Room, RoomExt, uuid::Uuid};
use proto::{RoomLeaveRequest, RoomLeaveResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use actix_multiresponse::Payload;
use tracing::instrument;

#[instrument]
pub async fn leave(data: WebData, payload: Payload<RoomLeaveRequest>) -> WebResult<Payload<RoomLeaveResponse>> {
    let mut room = match Room::get(data.dal.clone(), Uuid::parse_str(&payload.room_uuid)?)? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room does not exist"))
    };

    let resp = match room.remove_user(&Uuid::parse_str(&payload.user_uuid)?)? {
        RemoveStatus::LastMember => {
            room.delete()?;
            RoomLeaveResponse {
                deleted: true,
                new_owner: None,
            }
        },
        RemoveStatus::Ok { new_owner} =>  {
            RoomLeaveResponse {
                deleted: false,
                new_owner: Some(new_owner.to_string())
            }
        }
    };

    Ok(Payload(resp))
}