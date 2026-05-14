mod error;
mod send;
mod subscription;

use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};

use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use web_push::IsahcWebPushClient;

use crate::subscription::SubscriptionMap;

pub struct VapidConfig {
    pub private_key: String,
    pub subject: String,
}

#[derive(Clone)]
struct AppState {
    pub client: Arc<IsahcWebPushClient>,
    pub subscriptions: Arc<RwLock<SubscriptionMap>>,
    pub vapid: Arc<VapidConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let vapid = VapidConfig {
        private_key: std::env::var("VAPID_PRIVATE_KEY")
            .context("VAPID_PRIVATE_KEY must be set (see .env.example)")?,
        subject: std::env::var("VAPID_SUBJECT")
            .unwrap_or_else(|_| "mailto:test@example.com".into()),
    };

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let app = Router::new()
        .merge(send::router())
        .merge(subscription::router())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            client: Arc::new(IsahcWebPushClient::new()?),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            vapid: Arc::new(vapid),
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
