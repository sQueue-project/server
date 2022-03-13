use actix_web::web;
use dal::{Dal, User};
use dal::uuid::Uuid;
use proto::UserGetResponse;
use crate::appdata::WebData;
use crate::error::{Error, WebResult};
use actix_multiresponse::Payload;

pub async fn get(data: WebData, path: web::Path<Uuid>) -> WebResult<Payload<UserGetResponse>> {
    let user = match User::get(data.dal.clone(), path.into_inner())? {
        Some(x) => x,
        None => return Err(Error::NotFound("The requested user does not exist"))
    };

    Ok(Payload(UserGetResponse {
        name: user.name,
        uuid: user.uuid.to_string(),
    }))
}

