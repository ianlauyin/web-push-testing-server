use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use web_push::WebPushError;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(status = %self.status, message = %self.message, "api error");
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

impl From<WebPushError> for ApiError {
    fn from(e: WebPushError) -> Self {
        tracing::error!(
            kind = web_push_kind(&e),
            debug = ?e,
            display = %e,
            "web push send failed"
        );
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: format!("web_push {}: {e}", web_push_kind(&e)),
        }
    }
}

/// Static name of the `WebPushError` variant — easy to grep for in logs.
fn web_push_kind(e: &WebPushError) -> String {
    match e {
        WebPushError::Unspecified => "Unspecified".to_string(),
        WebPushError::Unauthorized => "Unauthorized".to_string(),
        WebPushError::NotImplemented => "NotImplemented".to_string(),
        WebPushError::InvalidUri => "InvalidUri".to_string(),
        WebPushError::EndpointNotValid => "EndpointNotValid".to_string(),
        WebPushError::EndpointNotFound => "EndpointNotFound".to_string(),
        WebPushError::PayloadTooLarge => "PayloadTooLarge".to_string(),
        WebPushError::TlsError => "TlsError".to_string(),
        WebPushError::SslError => "SslError".to_string(),
        WebPushError::IoError => "IoError".to_string(),
        WebPushError::InvalidPackageName => "InvalidPackageName".to_string(),
        WebPushError::InvalidTtl => "InvalidTtl".to_string(),
        WebPushError::InvalidTopic => "InvalidTopic".to_string(),
        WebPushError::MissingCryptoKeys => "MissingCryptoKeys".to_string(),
        WebPushError::InvalidCryptoKeys => "InvalidCryptoKeys".to_string(),
        WebPushError::InvalidResponse => "InvalidResponse".to_string(),
        WebPushError::InvalidClaims => "InvalidClaims".to_string(),
        WebPushError::BadRequest(None) => "BadRequest".to_string(),
        WebPushError::ServerError(None) => "ServerError".to_string(),
        WebPushError::BadRequest(Some(message)) => format!("BadRequest: {message}"),
        WebPushError::ServerError(Some(retry)) => format!("ServerError: retry after {:?}", retry),
        WebPushError::Other(message) => format!("Others: {message}"),
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }
    }
}
