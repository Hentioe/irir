use failure::Fail;
use libcore::errors::*;

use actix_web::{error, http, HttpResponse};

#[derive(Fail, Debug)]
pub enum WebError {
    #[fail(display = "Internal error, reason: {}", _0)]
    InternalError(String),
    #[fail(display = "Not Found")]
    NotFound,
}

impl WebError {
    pub fn internal(e: Error) -> Self {
        if let Some(e) = e.find_root_cause().downcast_ref::<std::io::Error>() {
            if e.kind() == std::io::ErrorKind::NotFound {
                return WebError::NotFound;
            }
        }
        WebError::InternalError(e.to_string())
    }

    pub fn parse(e: std::num::ParseIntError) -> Self {
        WebError::InternalError(e.to_string())
    }

    pub fn io(e: std::io::Error) -> Self {
        WebError::InternalError(e.to_string())
    }
}

impl error::ResponseError for WebError {
    fn error_response(&self) -> HttpResponse {
        match self {
            WebError::InternalError(_cause) => HttpResponse::with_body(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}", self),
            ),
            WebError::NotFound => HttpResponse::new(http::StatusCode::NOT_FOUND),
        }
    }
}
