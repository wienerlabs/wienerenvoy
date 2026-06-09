//! Service routes: Docker container and Compose-stack listing (M2.0, read-only).

use axum::Json;
use axum::extract::State;
use wienerenvoy_core::ServicesView;

use crate::error::ApiError;
use crate::state::AppState;

/// List Compose stacks and standalone containers. Returns an empty view (with
/// `docker.available = false`) when Docker is absent.
pub async fn get_services(State(state): State<AppState>) -> Result<Json<ServicesView>, ApiError> {
    if !state.config.permissions.allows("services") {
        return Err(ApiError::Conflict(
            "service control is disabled".to_string(),
        ));
    }
    Ok(Json(state.docker.services().await))
}
