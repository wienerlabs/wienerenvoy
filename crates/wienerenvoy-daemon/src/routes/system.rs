//! System telemetry routes.

use axum::Json;
use axum::extract::State;
use wienerenvoy_core::{SystemInfo, SystemSnapshot};

use crate::error::ApiError;
use crate::state::AppState;

/// Return the latest cached telemetry snapshot.
pub async fn get_metrics(State(state): State<AppState>) -> Result<Json<SystemSnapshot>, ApiError> {
    match state.snapshot.lock().await.clone() {
        Some(snapshot) => Ok(Json(snapshot)),
        None => Err(ApiError::Internal(
            "no telemetry sample available yet".to_string(),
        )),
    }
}

/// Return one-shot machine info.
pub async fn get_info(State(state): State<AppState>) -> Result<Json<SystemInfo>, ApiError> {
    Ok(Json(wienerenvoy_platform_macos::system_info(
        &state.version,
    )))
}
