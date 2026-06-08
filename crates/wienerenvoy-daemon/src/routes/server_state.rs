//! The server-level state route. Real in M0 because the state machine is
//! platform-agnostic and needs no power shellouts.

use axum::Json;
use axum::extract::State;
use wienerenvoy_core::ServerState;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_state(State(state): State<AppState>) -> Result<Json<ServerState>, ApiError> {
    let sm = state.state.lock().await;
    Ok(Json(sm.snapshot()))
}
