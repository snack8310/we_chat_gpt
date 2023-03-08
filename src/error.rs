use actix_web::{HttpResponse, ResponseError};

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::InvalidSignature => HttpResponse::BadRequest().body("Invalid signature"),
            Error::HttpError(e) => {
                HttpResponse::InternalServerError().body(format!("HTTP error: {}", e))
            }
            Error::DatabaseError(e) => {
                HttpResponse::InternalServerError().body(format!("Database error: {}", e))
            }
            Error::IoError(e) => {
                HttpResponse::InternalServerError().body(format!("io error: {}", e))
            }
            Error::JsonError(e) => {
                HttpResponse::InternalServerError().body(format!("json error: {}", e))
            }
        }
    }
}
