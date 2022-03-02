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
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dal(d) => {
                match d {
                    DalError::Mysql(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    DalError::Refinery(_) => unreachable!(),
                }
            },
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
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