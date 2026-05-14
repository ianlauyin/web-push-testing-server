use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, AppState};

pub type SubscriptionMap = HashMap<String, SubscriptionInfo>;

#[derive(Serialize, Clone)]
pub struct SubscriptionInfo {
    pub id: String,
    pub info: web_push::SubscriptionInfo,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/subscriptions", get(list).post(create))
        .route("/subscriptions/:id", delete(remove))
}

// ===== List all subscriptions endpoint =====
async fn list(State(state): State<AppState>) -> Json<SubscriptionMap> {
    let subs = state.subscriptions.read().await;
    Json(subs.clone())
}

// ===== Create subscription endpoint =====
#[derive(Serialize)]
pub struct CreateResponse {
    pub id: String,
}

async fn create(
    State(state): State<AppState>,
    Json(info): Json<web_push::SubscriptionInfo>,
) -> Result<Json<CreateResponse>, ApiError> {
    if info.endpoint.is_empty() || info.keys.p256dh.is_empty() || info.keys.auth.is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "missing endpoint or keys".into(),
        });
    }
    let mut subs = state.subscriptions.write().await;
    if let Some(existing) = subs.values().find(|s| s.info.endpoint == info.endpoint) {
        return Ok(Json(CreateResponse {
            id: existing.id.clone(),
        }));
    }
    let id = Uuid::new_v4().to_string();
    let info = SubscriptionInfo {
        id: id.clone(),
        info,
    };
    subs.insert(id.clone(), info.clone());
    tracing::info!("created subscription with id: {}", id);
    Ok(Json(CreateResponse { id }))
}

// ===== Remove subscription endpoint =====
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut subs = state.subscriptions.write().await;
    if subs.remove(&id).is_some() {
        tracing::info!("removed subscription with id: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "subscription not found".into(),
        })
    }
}
