//! Presence (keep-awake) routes.

use axum::Json;
use axum::extract::State;
use chrono::Utc;
use wienerenvoy_core::{KeepAwakeRequest, PresenceState, ServerState, WsFrame};

use crate::error::ApiError;
use crate::state::AppState;

fn presence_state(state: &AppState) -> PresenceState {
    PresenceState {
        keep_awake: state.keepawake.is_engaged(),
        backend: "caffeinate".to_string(),
        holder_pid: state.keepawake.holder_pid(),
        since: Utc::now(),
    }
}

pub async fn get_presence(State(state): State<AppState>) -> Result<Json<PresenceState>, ApiError> {
    Ok(Json(presence_state(&state)))
}

pub async fn post_keep_awake(
    State(state): State<AppState>,
    Json(req): Json<KeepAwakeRequest>,
) -> Result<Json<ServerState>, ApiError> {
    if !state.config.permissions.allows("presence") {
        return Err(ApiError::Conflict(
            "presence control is disabled".to_string(),
        ));
    }
    if req.enabled {
        state
            .keepawake
            .engage(req.reason.as_deref().unwrap_or("manual"))
            .await?;
    } else {
        state.keepawake.release().await?;
    }
    let _ = state
        .events
        .send(WsFrame::PresenceChanged(presence_state(&state)));
    let snapshot = state.state.lock().await.snapshot();
    Ok(Json(snapshot))
}
