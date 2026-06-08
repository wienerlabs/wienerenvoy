//! Liveness and bootstrap-probe handlers. Both are unauthenticated: `/health`
//! so launchd and the local CLI can probe, and the auth probe so the dashboard
//! can decide whether to prompt for a token.

use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub status: &'static str,
    pub version: String,
}

pub async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        version: state.version.clone(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProbe {
    pub needs_token: bool,
}

pub async fn auth_probe(State(_state): State<AppState>) -> Json<AuthProbe> {
    // M0 always uses bearer auth; M3 may add zero-password tailnet auth.
    Json(AuthProbe { needs_token: true })
}
