//! WebSocket telemetry route.
//!
//! Browsers cannot set WebSocket request headers, so the bearer token is passed
//! as a `?token=` query param and validated before the upgrade completes. After
//! upgrading, the socket receives a Hello, the latest snapshot, and then every
//! broadcast frame until it disconnects.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;
use wienerenvoy_core::WsFrame;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    #[serde(default)]
    token: String,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    if !state.auth.verify(&query.token) {
        return Err(ApiError::Unauthorized);
    }
    Ok(upgrade.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn send_frame(socket: &mut WebSocket, frame: &WsFrame) -> bool {
    match serde_json::to_string(frame) {
        Ok(text) => socket.send(Message::Text(text.into())).await.is_ok(),
        Err(_) => true, // skip a frame that fails to serialize, keep the socket
    }
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let hello = WsFrame::Hello {
        version: state.version.clone(),
        sample_interval_ms: state.config.metrics.sample_interval_ms,
    };
    if !send_frame(&mut socket, &hello).await {
        return;
    }
    if let Some(snapshot) = state.snapshot.lock().await.clone()
        && !send_frame(&mut socket, &WsFrame::Metrics(snapshot)).await
    {
        return;
    }

    let mut rx = state.events.subscribe();
    loop {
        tokio::select! {
            received = rx.recv() => match received {
                Ok(frame) => {
                    if !send_frame(&mut socket, &frame).await {
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => {} // dropped frames on a slow client; keep going
                Err(RecvError::Closed) => break,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}      // ignore client chatter
                Some(Err(_)) => break,
            },
        }
    }
}
