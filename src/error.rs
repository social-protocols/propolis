use axum::response::{IntoResponse, Response};
use http::StatusCode;

/// Our custom wrapper type for various errors, so we can implement IntoResponse for e.g. sqlx::Error
pub enum Error {
    SqlxError(sqlx::Error),
    IoError(std::io::Error),
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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self {
            Error::SqlxError(sqlx_error) => sqlx_error.to_string(),
            Error::IoError(io_error) => io_error.to_string(),
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
