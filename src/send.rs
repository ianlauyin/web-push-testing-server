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
#[serde(rename_all = "camelCase")]
struct SendRequest {
    id: String,
    /// Push payload format. Defaults to `LEGACY`.
    #[serde(default)]
    push_type: PushType,
    payload: Payload,
}

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Deserialize, Default, Clone, Copy, Debug, PartialEq)]
enum PushType {
    /// Traditional encrypted payload — service worker decrypts and calls
    /// `showNotification(...)`.
    #[default]
    LEGACY,
    /// iOS 18.4+ Declarative Web Push — server wraps payload as
    /// `{ web_push: 8030, notification: { title, body, navigate } }`.
    DECLARATIVE,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    pub title: String,
    pub body: String,
    /// Required when `push_type = declarative`. URL the OS opens on tap.
    #[serde(default)]
    pub navigate: Option<String>,
    /// Consumed by the service worker (only when `push_type = legacy`).
    #[serde(default)]
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

// Declarative Web Push envelope (iOS 18.4+).
// https://developer.apple.com/documentation/usernotifications/sending-web-push-notifications-in-web-apps-and-browsers
#[derive(Serialize)]
struct DeclarativeEnvelope<'a> {
    web_push: u32,
    notification: DeclarativeNotification<'a>,
}

#[derive(Serialize)]
struct DeclarativeNotification<'a> {
    title: &'a str,
    body: &'a str,
    navigate: &'a str,
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

    let payload_bytes = build_payload(req.push_type, &req.payload)?;

    let mut sig = VapidSignatureBuilder::from_base64(
        &state.vapid.private_key,
        URL_SAFE_NO_PAD,
        &subscription.info,
    )?;
    sig.add_claim("sub", state.vapid.subject.as_str());

    let mut msg = WebPushMessageBuilder::new(&subscription.info);
    msg.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
    msg.set_vapid_signature(sig.build()?);

    state.client.send(msg.build()?).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn build_payload(push_type: PushType, payload: &Payload) -> Result<Vec<u8>, ApiError> {
    match push_type {
        PushType::LEGACY => Ok(serde_json::to_vec(payload)?),
        PushType::DECLARATIVE => {
            let navigate = payload.navigate.as_deref().ok_or(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: "declarative push requires `payload.navigate`".into(),
            })?;
            let envelope = DeclarativeEnvelope {
                web_push: 8030,
                notification: DeclarativeNotification {
                    title: &payload.title,
                    body: &payload.body,
                    navigate,
                },
            };
            Ok(serde_json::to_vec(&envelope)?)
        }
    }
}
