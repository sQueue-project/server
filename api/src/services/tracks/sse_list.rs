use crate::appdata::WebData;
use crate::error::WebResult;
use crate::services::sse::{Broadcaster, SseResponse};
use actix_web::web;
use dal::uuid::Uuid;

pub async fn sse_list(data: WebData, path: web::Path<Uuid>) -> WebResult<SseResponse> {
    let mut map = data.sse_track_list.lock();
    let am_broadcaster = map.entry(path.into_inner())
        .or_insert(Broadcaster::new());
    let mut broadcaster = am_broadcaster.lock();

    let new_client = broadcaster.new_client()?;
    Ok(new_client.into())
}