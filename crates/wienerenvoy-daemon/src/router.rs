//! Router assembly: public routes, bearer-gated routes, the WebSocket route
//! (self-authenticated), the static dashboard fallback, and the global
//! peer-guard plus trace layers.

use std::path::PathBuf;

use axum::routing::{get, post};
use axum::{Router, middleware};
use tower_http::trace::TraceLayer;

use crate::auth::{peer_guard, require_bearer};
use crate::routes;
use crate::state::AppState;
use crate::static_assets;

/// Build the full application router.
pub fn build_router(state: AppState, web_dir: Option<PathBuf>) -> Router {
    // Routes that require a valid bearer token in the Authorization header.
    let protected = Router::new()
        .route("/api/v1/state", get(routes::server_state::get_state))
        .route("/api/v1/system/metrics", get(routes::system::get_metrics))
        .route("/api/v1/system/info", get(routes::system::get_info))
        .route("/api/v1/presence", get(routes::presence::get_presence))
        .route(
            "/api/v1/presence/keep-awake",
            post(routes::presence::post_keep_awake),
        )
        .route("/api/v1/power/server/off", post(routes::power::server_off))
        .route("/api/v1/power/server/on", post(routes::power::server_on))
        .route("/api/v1/power/sleep", post(routes::power::sleep))
        .route("/api/v1/power/restart", post(routes::power::restart))
        .route("/api/v1/power/shutdown", post(routes::power::shutdown))
        .route(
            "/api/v1/power/wake-schedule",
            post(routes::power::wake_schedule),
        )
        // Service-level control arrives in M2.
        .route("/api/v1/services", get(routes::stub::not_implemented))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_bearer));

    // The WebSocket route authenticates via ?token= inside the handler, because
    // browsers cannot set request headers on a WebSocket.
    let ws = Router::new().route("/api/v1/ws", get(routes::ws::ws_handler));

    // Public routes (no auth): liveness and a bootstrap probe.
    let public = Router::new()
        .route("/health", get(routes::health::health))
        .route("/api/v1/auth/probe", get(routes::health::auth_probe));

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(ws)
        .merge(static_assets::ui_router(web_dir))
        .layer(middleware::from_fn(peer_guard))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
