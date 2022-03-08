use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use crate::appdata::WebData;
use crate::error::WebResult;
use crate::services::sse::{Broadcaster, SseResponse};
use actix_web::{FromRequest, HttpRequest, web};
use actix_web::dev::Payload;
use tracing::trace;
use dal::uuid::Uuid;
use crate::services::payload::ContentType;

pub struct XAccept(ContentType);

impl Deref for XAccept {
    type Target = ContentType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for XAccept {
    type Error = crate::error::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let ct = ContentType::from_request_header(&req, "X-Accept");
            if ct.eq(&ContentType::Other) {
                Err(crate::error::Error::BadRequest("Bad value for header 'X-Accept'"))
            } else {
                Ok(Self(ct))
            }
        })
    }
}
pub async fn sse_list(data: WebData, path: web::Path<Uuid>, xaccept: XAccept) -> WebResult<SseResponse> {
    let mut map = data.sse_track_list.lock();
    let uuid = path.into_inner();

    let client = if map.contains_key(&uuid) {
        trace!("Map already has Broadcaster for UUID {uuid}");

        let am_broadcaster = map.get_mut(&uuid).unwrap();
        let mut broadcaster = am_broadcaster.lock();
        let new_client = broadcaster.new_client(xaccept.clone())?;

        new_client
    } else {
        trace!("No Broadcaster exists for UUID {uuid}, creating");

        let am_broadcaster = Broadcaster::new(uuid.clone());
        let mut broadcaster = am_broadcaster.lock();
        let new_client = broadcaster.new_client(xaccept.clone())?;

        drop(broadcaster);
        map.insert(uuid, am_broadcaster);

        new_client
    };
    
    Ok(client.into())
}