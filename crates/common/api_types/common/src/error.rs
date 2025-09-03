use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Api Endpoint Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Internal Server Error: {0}")]
    InternalError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),

    #[error("Too many validator IDs in request")]
    TooManyValidatorsIds,

    #[error("Node is currently syncing and not serving request on that endpoint")]
    UnderSyncing,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
            ApiError::ValidatorNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::TooManyValidatorsIds => StatusCode::URI_TOO_LONG,
            ApiError::UnderSyncing => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
