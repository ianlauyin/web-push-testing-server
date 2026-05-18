use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use web_push::{
    ContentEncoding, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder, URL_SAFE_NO_PAD,
};

use crate::{error::ApiError, AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/send", post(send))
}

#[derive(Deserialize)]
struct SendRequest {
    id: String,
    payload: Payload,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    pub title: String,
    pub body: String,
    pub redirection: Option<Redirection>,
}

#[derive(Serialize, Deserialize)]
struct Redirection {
    pub r#type: PushNotificationRedirectTypeView,
    pub id: Option<String>,
}

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Serialize, Deserialize)]
enum PushNotificationRedirectTypeView {
    ANNOUNCEMENT,
    CAMPAIGN,
    CHAT,
    DEPOSIT,
    WITHDRAWAL,
    CUSTOMER_SERVICE,
}

async fn send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<StatusCode, ApiError> {
    let subs = state.subscriptions.read().await;
    let subscription = subs.get(&req.id).ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "subscription not found".into(),
    })?;

    let mut sig = VapidSignatureBuilder::from_base64(
        &state.vapid.private_key,
        URL_SAFE_NO_PAD,
        &subscription.info,
    )?;
    sig.add_claim("sub", state.vapid.subject.as_str());

    let mut msg = WebPushMessageBuilder::new(&subscription.info);
    let payload = serde_json::to_vec(&req.payload)?;
    msg.set_payload(ContentEncoding::Aes128Gcm, &payload);
    msg.set_vapid_signature(sig.build()?);

    state.client.send(msg.build()?).await?;
    Ok(StatusCode::NO_CONTENT)
}
