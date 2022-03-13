use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use crate::error::Error;

/// Possible values for the content type supported in this server
#[derive(Debug, Clone)]
pub enum ContentType {
    Json,
    Protobuf
}

/// The value in the `X-Accept` header
#[derive(Debug, Clone)]
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
            if let Some(content_type) = get_content_type_x_accept(&req) {
                Ok(Self(content_type))
            } else {
                Err(Error::BadRequest("Bad value or missing value for header 'X-Accept'"))
            }
        })
    }
}

/// Get the content type provided in the `X-Accept` header, if any
fn get_content_type_x_accept(req: &HttpRequest) -> Option<ContentType> {
    let x_accept = get_header_x_accept(req)?.to_lowercase();

    if x_accept.starts_with("application/json") {
        Some(ContentType::Json)
    } else if x_accept.starts_with("application/protobuf") {
        Some(ContentType::Protobuf)
    } else {
        None
    }
}

/// Get the string value of the `X-Accept` header
fn get_header_x_accept(req: &HttpRequest) -> Option<&str> {
    let header_value = req.headers().get("X-Accept")?;
    let as_string = header_value.to_str().ok()?;
    Some(as_string)
}