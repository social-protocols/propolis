//! Error handling related code

use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Our custom wrapper type for various errors, so we can implement IntoResponse for e.g. sqlx::Error
#[derive(Debug)]
pub enum Error {
    Sqlx(sqlx::Error),
    Io(std::io::Error),
    Anyhow(anyhow::Error),
    Custom(String),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Error {
        Error::Sqlx(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Error {
        Error::Anyhow(err)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        match err {
            Error::Sqlx(sqlx_error) => sqlx_error.to_string(),
            Error::Io(io_error) => io_error.to_string(),
            Error::Anyhow(error) => error.to_string(),
            Error::Custom(msg) => msg,
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
            format!("INTERNAL SERVER ERROR:\n\n{error_msg}"),
        )
            .into_response()
    }
}
