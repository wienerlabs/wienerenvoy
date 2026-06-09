//! Service routes: Docker container and Compose-stack listing, lifecycle
//! control, and log tailing.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use wienerenvoy_core::{ControlResult, LogLines, ServiceControl, ServiceTarget, ServicesView};

use crate::error::ApiError;
use crate::state::AppState;

fn ensure_services(state: &AppState) -> Result<(), ApiError> {
    if state.config.permissions.allows("services") {
        Ok(())
    } else {
        Err(ApiError::Conflict(
            "service control is disabled".to_string(),
        ))
    }
}

/// List Compose stacks and standalone containers. Returns an empty view (with
/// `docker.available = false`) when Docker is absent.
pub async fn get_services(State(state): State<AppState>) -> Result<Json<ServicesView>, ApiError> {
    ensure_services(&state)?;
    Ok(Json(state.docker.services().await))
}

/// Start, stop, or restart a stack or a single container.
pub async fn post_control(
    State(state): State<AppState>,
    Json(req): Json<ServiceControl>,
) -> Result<Json<ControlResult>, ApiError> {
    ensure_services(&state)?;
    let affected = match req.kind {
        ServiceTarget::Stack => state.docker.stack_action(&req.id, req.action).await?,
        ServiceTarget::Container => {
            state.docker.container_action(&req.id, req.action).await?;
            1
        }
    };
    Ok(Json(ControlResult { ok: true, affected }))
}

#[derive(Deserialize)]
pub struct LogsQuery {
    id: String,
    #[serde(default = "default_tail")]
    tail: usize,
}

fn default_tail() -> usize {
    200
}

/// Tail a container's logs.
pub async fn get_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<LogLines>, ApiError> {
    ensure_services(&state)?;
    let lines = state.docker.container_logs(&query.id, query.tail).await?;
    Ok(Json(LogLines { lines }))
}
