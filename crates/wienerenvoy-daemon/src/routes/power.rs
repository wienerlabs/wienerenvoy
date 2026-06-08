//! Power routes: server-level on/off and machine-level sleep/restart/shutdown.
//!
//! The sleep-default / shutdown-confirm asymmetry lives here: sleep needs no
//! confirmation (recoverable over the tailnet), restart and shutdown do (they
//! can lock the operator out). Each action is checked against the configured
//! allowlist first.

use std::time::Duration;

use axum::Json;
use axum::extract::State;
use chrono::Utc;
use wienerenvoy_core::{
    ActionAccepted, ConfirmedAction, MachineCommand, ServerLevel, ServerState, ServerStateMachine,
    WakeRequest, WakeSpec, WsFrame,
};

use crate::error::ApiError;
use crate::state::AppState;

fn ensure_allowed(state: &AppState, action: &str) -> Result<(), ApiError> {
    if state.config.permissions.allows(action) {
        Ok(())
    } else {
        Err(ApiError::Conflict(format!(
            "action '{action}' is disabled by configuration"
        )))
    }
}

fn accepted(action: &str, cmd: MachineCommand, warning: Option<String>) -> ActionAccepted {
    ActionAccepted {
        accepted: true,
        action: action.to_string(),
        recoverable_via: ServerStateMachine::recovery_for(cmd),
        warning,
    }
}

pub async fn server_off(State(state): State<AppState>) -> Result<Json<ServerState>, ApiError> {
    ensure_allowed(&state, "server_off")?;
    state.keepawake.release().await?;
    let snapshot = {
        let mut sm = state.state.lock().await;
        sm.apply_level(ServerLevel::ServerOff, Utc::now())
    };
    let _ = state.events.send(WsFrame::StateChanged(snapshot.clone()));
    Ok(Json(snapshot))
}

pub async fn server_on(State(state): State<AppState>) -> Result<Json<ServerState>, ApiError> {
    ensure_allowed(&state, "server_off")?;
    state.keepawake.engage("server on").await?;
    let snapshot = {
        let mut sm = state.state.lock().await;
        sm.apply_level(ServerLevel::Active, Utc::now())
    };
    let _ = state.events.send(WsFrame::StateChanged(snapshot.clone()));
    Ok(Json(snapshot))
}

pub async fn sleep(State(state): State<AppState>) -> Result<Json<ActionAccepted>, ApiError> {
    ensure_allowed(&state, "sleep")?;
    state.power.sleep().await?;
    Ok(Json(accepted("sleep", MachineCommand::Sleep, None)))
}

pub async fn restart(
    State(state): State<AppState>,
    Json(req): Json<ConfirmedAction>,
) -> Result<Json<ActionAccepted>, ApiError> {
    ensure_allowed(&state, "restart")?;
    if MachineCommand::Restart.requires_confirm() && !req.confirm {
        return Err(ApiError::Conflict(
            "restart requires confirmation".to_string(),
        ));
    }
    state
        .power
        .restart(Duration::from_secs(req.grace_secs))
        .await?;
    Ok(Json(accepted("restart", MachineCommand::Restart, None)))
}

pub async fn shutdown(
    State(state): State<AppState>,
    Json(req): Json<ConfirmedAction>,
) -> Result<Json<ActionAccepted>, ApiError> {
    ensure_allowed(&state, "shutdown")?;
    if !req.confirm {
        return Err(ApiError::Conflict(
            "shutdown requires confirmation".to_string(),
        ));
    }
    state
        .power
        .shutdown(Duration::from_secs(req.grace_secs))
        .await?;
    Ok(Json(accepted(
        "shutdown",
        MachineCommand::Shutdown,
        Some("the tailnet goes down on shutdown; recover via Wake-on-LAN only".to_string()),
    )))
}

pub async fn wake_schedule(
    State(state): State<AppState>,
    Json(req): Json<WakeRequest>,
) -> Result<Json<ActionAccepted>, ApiError> {
    ensure_allowed(&state, "sleep")?;
    let spec = if let Some(secs) = req.relative_secs {
        WakeSpec::Relative { secs }
    } else if let Some(at) = req.at {
        WakeSpec::Schedule { at }
    } else {
        return Err(ApiError::Conflict(
            "wake request needs relativeSecs or at".to_string(),
        ));
    };
    state.power.schedule_wake(spec).await?;
    Ok(Json(ActionAccepted {
        accepted: true,
        action: "wake-schedule".to_string(),
        recoverable_via: Vec::new(),
        warning: None,
    }))
}
