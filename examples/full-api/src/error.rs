use autometrics::AutometricsLabel;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

// We're using the `AutometricsLabel` derive to enable the error variants to be turned into labels
// on the call counter metric.
//
// In this case, the label will be `error` = `not_found`, `bad_request`, or `internal_server_error`.
//
// Instead of looking at high-level HTTP status codes in our metrics,
// we'll instead see the actual variant name of the error.
#[derive(Debug, AutometricsLabel)]
#[autometrics_label(key = "error")]
pub enum ApiError {
    #[autometrics_label()]
    NotFound,
    #[autometrics_label()]
    BadRequest,
    #[autometrics_label(value = "internal_server_error")]
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
