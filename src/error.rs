//! Error handling related code

use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Our custom wrapper type for various errors, so we can implement IntoResponse for e.g. sqlx::Error
#[derive(Debug)]
pub enum Error {
    Anyhow(anyhow::Error),
    Custom(String),
    Io(std::io::Error),
    Serde(serde_json::Error),
    Sqlx(sqlx::Error),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Error {
        Error::Anyhow(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Error {
        Error::Sqlx(err)
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        match err {
            Error::Anyhow(error) => error.to_string(),
            Error::Custom(msg) => msg,
            Error::Io(error) => error.to_string(),
            Error::Serde(error) => error.to_string(),
            Error::Sqlx(error) => error.to_string(),
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
