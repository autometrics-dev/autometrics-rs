use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use strum::IntoStaticStr;
use thiserror::Error;

// We're using `thiserror` to define our error type, and we're using `strum` to
// enable the error variants to be turned into &'static str's, which
// will actually become another label on the call counter metric.
//
// In this case, the label will be `error` = `not_found`, `bad_request`, or `internal`.
//
// Instead of looking at high-level HTTP status codes in our metrics,
// we'll instead see the actual variant name of the error.
#[derive(Debug, Error, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ApiError {
    #[error("User not found")]
    NotFound,
    #[error("Bad request")]
    BadRequest,
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, format!("{:?}", self)).into_response()
    }
}
