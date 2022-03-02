use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use actix_protobuf::ProtoBuf;
use actix_web::{FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder, ResponseError, web};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use tracing::trace;

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Payload error: {0}")]
    ActixPayload(#[from] actix_web::error::PayloadError),
    #[error("Error: {0}")]
    Actix(#[from] actix_web::Error),
    #[error("Invalid content type")]
    InvalidContentType,
}

impl ResponseError for PayloadError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .body(format!("{self}"))
    }
}

#[derive(Debug)]
pub struct Payload<T: 'static + DeserializeOwned + Message + Default + Clone>(T);

impl<T: 'static + DeserializeOwned + Message + Default + Clone> Deref for Payload<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static + DeserializeOwned + Message + Default + Clone> FromRequest for Payload<T> {
    type Error = PayloadError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = payload.take();

        Box::pin(async move {
            let data = match ContentType::from_request(&req) {
                ContentType::Json => {
                    trace!("Received JSON payload, deserializing");
                    let json: web::Json<T> = web::Json::from_request(&req, &mut payload).await?;
                    Self(json.clone())
                },
                ContentType::Protobuf => {
                    trace!("Received Protobuf payload, deserializing");
                    let protobuf: ProtoBuf<T> = ProtoBuf::from_request(&req, &mut payload).await?;
                    Self(protobuf.clone())
                },
                _ => {
                    trace!("User did not set a valid Content-Type header");
                    return Err(Self::Error::InvalidContentType)
                }
            };

            Ok(data)
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ContentType {
    Json,
    Protobuf,
    Other,
}

impl ContentType {
    #[inline]
    fn from_request(req: &HttpRequest) -> Self {
        match req.content_type() {
            "application/json" => Self::Json,
            "application/protobuf" => Self::Protobuf,
            _ => Self::Other,
        }
    }
}

#[derive(Debug)]
pub struct TypedResponse<T: Serialize + Message + Default + Debug>(pub T);

impl<T: Serialize + Message + Default> Responder for TypedResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let content_type = if let Some(accepts) = req.headers().get("Accepts") {
            if let Ok(accepts) = accepts.to_str() {
                match accepts {
                    "application/json" => ContentType::Json,
                    "application/protobuf" => ContentType::Protobuf,
                    _ => ContentType::from_request(req)
                }
            } else {
                ContentType::from_request(req)
            }
        } else {
            ContentType::from_request(req)
        };

        let content_type = if content_type.eq(&ContentType::Other) {
            ContentType::Json
        } else {
            content_type
        };

        match content_type {
            ContentType::Json => {
                let json = web::Json(self.0);
                json.respond_to(req).map_into_boxed_body()
            },
            ContentType::Protobuf => {
                let protobuf = ProtoBuf(self.0);
                protobuf.respond_to(req)
            },
            ContentType::Other => unreachable!()
        }
    }
}