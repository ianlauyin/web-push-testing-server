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
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

impl From<WebPushError> for ApiError {
    fn from(e: WebPushError) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: e.to_string(),
        }
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
