//! Router assembly: public routes, bearer-gated routes, the static dashboard
//! fallback, and the global peer-guard plus trace layers.

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
    // Routes that require a valid bearer token.
    let protected = Router::new()
        .route("/api/v1/state", get(routes::server_state::get_state))
        .route("/api/v1/system/metrics", get(routes::stub::not_implemented))
        .route("/api/v1/system/info", get(routes::stub::not_implemented))
        .route("/api/v1/presence", get(routes::stub::not_implemented))
        .route(
            "/api/v1/presence/keep-awake",
            post(routes::stub::not_implemented),
        )
        .route(
            "/api/v1/power/server/off",
            post(routes::stub::not_implemented),
        )
        .route(
            "/api/v1/power/server/on",
            post(routes::stub::not_implemented),
        )
        .route("/api/v1/power/sleep", post(routes::stub::not_implemented))
        .route("/api/v1/power/restart", post(routes::stub::not_implemented))
        .route(
            "/api/v1/power/shutdown",
            post(routes::stub::not_implemented),
        )
        .route(
            "/api/v1/power/wake-schedule",
            post(routes::stub::not_implemented),
        )
        .route("/api/v1/services", get(routes::stub::not_implemented))
        .route("/api/v1/ws", get(routes::stub::not_implemented))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_bearer,
        ));

    // Public routes (no auth): liveness and a bootstrap probe.
    let public = Router::new()
        .route("/health", get(routes::health::health))
        .route("/api/v1/auth/probe", get(routes::health::auth_probe));

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(static_assets::ui_router(web_dir))
        .layer(middleware::from_fn(peer_guard))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
