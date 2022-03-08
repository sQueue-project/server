use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use crate::appdata::WebData;
use crate::error::WebResult;
use crate::services::sse::{Broadcaster, SseResponse};
use actix_web::{FromRequest, HttpRequest, web};
use actix_web::dev::Payload;
use dal::uuid::Uuid;
use crate::services::payload::ContentType;

pub struct XAccepts(ContentType);

impl Deref for XAccepts {
    type Target = ContentType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for XAccepts {
    type Error = crate::error::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let ct = ContentType::from_request_header(&req, "X-Accepts");
            if ct.eq(&ContentType::Other) {
                Err(crate::error::Error::BadRequest("Bad value for header 'X-Accepts'"))
            } else {
                Ok(Self(ct))
            }
        })
    }
}
pub async fn sse_list(data: WebData, path: web::Path<Uuid>, xaccepts: XAccepts) -> WebResult<SseResponse> {
    let mut map = data.sse_track_list.lock();
    let am_broadcaster = map.entry(path.into_inner())
        .or_insert(Broadcaster::new());
    let mut broadcaster = am_broadcaster.lock();

    let new_client = broadcaster.new_client(xaccepts.clone())?;
    Ok(new_client.into())
}