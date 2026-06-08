//! Serving the dashboard. In M0 (and dev) the daemon serves the static export
//! from `WIENERENVOY_WEB_DIR` via `ServeDir`. The release build will bake the
//! export into the binary behind the `embed-ui` feature; until then, an
//! unbuilt dashboard falls back to a small placeholder so the API is still
//! discoverable.

use std::path::PathBuf;

use axum::Router;
use axum::response::Html;

use crate::state::AppState;

/// Build the UI sub-router that handles all non-API paths.
pub fn ui_router(web_dir: Option<PathBuf>) -> Router<AppState> {
    match web_dir {
        Some(dir) if dir.is_dir() => {
            use tower_http::services::ServeDir;
            let serve = ServeDir::new(&dir).append_index_html_on_directories(true);
            tracing::info!(dir = %dir.display(), "serving dashboard from disk");
            Router::new().fallback_service(serve)
        }
        _ => {
            tracing::info!("no web dir configured; serving placeholder page");
            Router::new().fallback(placeholder)
        }
    }
}

async fn placeholder() -> Html<&'static str> {
    Html(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>WienerEnvoy</title></head>\
         <body style=\"background:#07070a;color:#fafafa;font-family:ui-sans-serif,system-ui,sans-serif;\
         margin:0;display:flex;min-height:100vh;align-items:center;justify-content:center\">\
         <div style=\"max-width:38rem;padding:2rem\">\
         <h1 style=\"font-weight:300;letter-spacing:-0.02em\">WienerEnvoy</h1>\
         <p style=\"color:#a1a1aa;line-height:1.6\">The daemon is live. The API responds at \
         <code style=\"color:#34d399\">/health</code>. The dashboard is not bundled in this build. \
         Run <code>pnpm -C web build</code> and set <code>WIENERENVOY_WEB_DIR</code>, or build the \
         daemon with <code>--features embed-ui</code>.</p>\
         </div></body></html>",
    )
}
