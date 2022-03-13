use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use thiserror::Error;
use dal::Error as DalError;

pub type WebResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
#[allow(unused)]
pub enum Error {
    #[error("Dal error: {0}")]
    Dal(#[from] dal::Error),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Not found: {0}")]
    NotFound(&'static str),
    #[error("Bad request: {0}")]
    BadRequest(&'static str),
    #[error("UUID error: {0}")]
    Uuid(#[from] dal::uuid::Error),
    #[error("Sse: {0}")]
    Sse(#[from] crate::services::sse::broadcaster::SseError),
    #[error("Forbidden: {0}")]
    Forbidden(&'static str),
    #[error("Requwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dal(d) => {
                match d {
                    DalError::Mysql(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    DalError::Refinery(_) => unreachable!(),
                    DalError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            },
            Self::Sse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) | Self::Uuid(_) => StatusCode::BAD_REQUEST,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Reqwest(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TokioJoin(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = if self.status_code().is_client_error() {
            format!("{self}")
        } else {
            self.status_code().canonical_reason().unwrap_or("").to_string()
        };

        HttpResponse::build(self.status_code())
            .body(body)
    }
}