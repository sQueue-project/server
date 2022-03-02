use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use actix_web::http::StatusCode;
use thiserror::Error;
use dal::Error as DalError;

pub type WebResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dal error: {0}")]
    Dal(#[from] dal::Error)
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dal(d) => {
                match d {
                    DalError::Mysql(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        }
    }
}