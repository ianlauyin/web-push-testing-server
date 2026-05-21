use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, post},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, AppState};

pub type SubscriptionMap = HashMap<String, SubscriptionInfo>;

#[derive(Serialize, Clone, Debug)]
pub struct SubscriptionInfo {
    pub id: String,
    pub info: web_push::SubscriptionInfo,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/subscriptions", post(create).get(list))
        .route("/subscriptions/:id", delete(remove))
}

// ===== List subscriptions endpoint =====
async fn list(State(state): State<AppState>) -> Json<Vec<SubscriptionInfo>> {
    let subs = state.subscriptions.read().await;
    Json(subs.values().cloned().collect())
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
    tracing::info!("subscription: {:?}", info.clone());
    Ok(Json(CreateResponse { id }))
}

// ===== Remove subscription endpoint =====
async fn remove(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let state_clone = state.clone();
    if remove_by_id(state_clone, id).await {
        return Ok(StatusCode::NO_CONTENT);
    }
    Err(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "subscription not found".into(),
    })
}

pub async fn remove_by_id(state: AppState, id: String) -> bool {
    let mut subs = state.subscriptions.write().await;
    if subs.remove(&id).is_some() {
        tracing::info!("removed subscription with id: {}", id);
        true
    } else {
        false
    }
}
