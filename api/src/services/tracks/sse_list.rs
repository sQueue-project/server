use crate::appdata::WebData;
use crate::error::WebResult;
use crate::services::sse::broadcaster::{Broadcaster, SseResponse};
use actix_web::web;
use tracing::trace;
use dal::uuid::Uuid;
use crate::services::sse::x_accept::XAccept;

pub async fn sse_list(data: WebData, path: web::Path<Uuid>, xaccept: XAccept) -> WebResult<SseResponse> {
    let mut map = data.sse_track_list.lock();
    let uuid = path.into_inner();

    let client = if map.contains_key(&uuid) {
        trace!("Map already has Broadcaster for UUID {uuid}");

        let am_broadcaster = map.get_mut(&uuid).unwrap();
        let mut broadcaster = am_broadcaster.lock();
        let new_client = broadcaster.new_client((*xaccept).clone())?;

        new_client
    } else {
        trace!("No Broadcaster exists for UUID {uuid}, creating");

        let am_broadcaster = Broadcaster::new(uuid.clone());
        let mut broadcaster = am_broadcaster.lock();
        let new_client = broadcaster.new_client((*xaccept).clone())?;

        drop(broadcaster);
        map.insert(uuid, am_broadcaster);

        new_client
    };
    
    Ok(client.into())
}