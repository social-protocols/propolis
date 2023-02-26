//! Error handling related code

use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Our custom wrapper type for various errors, so we can implement IntoResponse for e.g. sqlx::Error
#[derive(Debug)]
pub enum Error {
    SqlxError(sqlx::Error),
    IoError(std::io::Error),
    AskamaError(askama::Error),
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

impl From<askama::Error> for Error {
    fn from(err: askama::Error) -> Error {
        Error::AskamaError(err)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        match err {
            Error::SqlxError(sqlx_error) => sqlx_error.to_string(),
            Error::IoError(io_error) => io_error.to_string(),
            Error::AskamaError(error) => error.to_string(),
        }
    }
}

/// Support custom [Error] as an axum Response
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, String::from(self)).into_response()
    }
}
