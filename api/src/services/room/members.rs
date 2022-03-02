use actix_web::web;
use dal::{Dal, Room, RoomExt, User};
use dal::uuid::Uuid;
use proto::{RoomMember, RoomMemberResponse};
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use crate::services::payload::TypedResponse;
use tracing::instrument;

#[instrument]
pub async fn members(data: WebData, path: web::Path<Uuid>) -> WebResult<TypedResponse<RoomMemberResponse>> {
    let room = match Room::get(data.dal.clone(), path.into_inner())? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested room does not exist"))
    };

    let members = room.list_members()?
        .into_iter()
        .map(|m| {
            let user = User::get(data.dal.clone(), m.uuid.clone())?
                .ok_or(Error::Conflict(format!("Cant find user {}", m.uuid)))?;
            Ok(RoomMember {
                uuid: m.uuid.to_string(),
                name: user.name,
                owner: room.owner.eq(&user.uuid),
                joined_at: m.joined_at
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(TypedResponse(RoomMemberResponse {
        members
    }))
}