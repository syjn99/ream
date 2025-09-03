use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("Request failed with status code: {status_code}")]
    RequestFailed { status_code: reqwest::StatusCode },

    #[error("Request failed with status code: {status_code}, message: {message}")]
    RequestFailedWithMessage {
        status_code: reqwest::StatusCode,
        message: String,
    },

    #[error("Failed to decode SSZ response: {0}")]
    SszDecodeError(String),

    #[error("Failed to decode JSON response: {0}")]
    JsonDecodeError(String),

    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Invalid content type: expected application/octet-stream or application/json")]
    InvalidContentType,

    #[error("Network timeout")]
    Timeout,

    #[error("Invalid response format")]
    InvalidResponse,

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
