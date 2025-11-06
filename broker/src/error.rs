use axum::{http::StatusCode, response::IntoResponse};
use tokio::io;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError,
    SawnTimeOut,
    NoFreePorts,
    NotFound,
    LightYearError(lightyear::netcode::Error)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        log::error!("{:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<lightyear::netcode::Error> for Error {
    fn from(value: lightyear::netcode::Error) -> Self {
        Self::LightYearError(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}
