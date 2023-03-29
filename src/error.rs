//! Error handling related code

use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Our custom wrapper type for various errors, so we can implement IntoResponse for e.g. sqlx::Error
#[derive(Debug)]
pub enum Error {
    SqlxError(sqlx::Error),
    IoError(std::io::Error),
    AnyhowError(anyhow::Error),
    CustomError(String),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Error {
        Error::SqlxError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Error {
        Error::AnyhowError(err)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        match err {
            Error::SqlxError(sqlx_error) => sqlx_error.to_string(),
            Error::IoError(io_error) => io_error.to_string(),
            Error::AnyhowError(error) => error.to_string(),
            Error::CustomError(msg) => msg,
        }
    }
}

/// Support custom [Error] as an axum Response
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let error_msg = String::from(self);
        tracing::error!(error_msg);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("INTERNAL SERVER ERROR:\n\n{}", error_msg),
        )
            .into_response()
    }
}
