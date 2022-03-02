use actix_protobuf::ProtoBuf;
use proto::{CreateRoomRequest, CreatedResponse};
use crate::appdata::WebData;
use crate::error::WebResult;

pub async fn create(data: WebData, payload: ProtoBuf<CreateRoomRequest>) -> WebResult<CreatedResponse> {
    unimplemented!()
}
