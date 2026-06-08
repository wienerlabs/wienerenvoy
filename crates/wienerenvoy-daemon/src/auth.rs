//! Two middleware layers form the security boundary:
//!
//!   - `peer_guard`: a global layer that rejects any connection whose peer is
//!     not loopback or inside the Tailscale CGNAT range. Belt-and-suspenders on
//!     top of the bind restriction.
//!   - `require_bearer`: gates the protected routes on a constant-time bearer
//!     token check.

use std::net::{IpAddr, SocketAddr};

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{Request, header};
use axum::middleware::Next;
use axum::response::Response;
use wienerenvoy_core::tailscale::is_cgnat;

use crate::error::ApiError;
use crate::state::AppState;

/// Reject connections from outside loopback or the tailnet.
pub async fn peer_guard(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let allowed = match peer.ip() {
        IpAddr::V4(v4) => v4.is_loopback() || is_cgnat(v4),
        IpAddr::V6(v6) => v6.is_loopback(),
    };
    if !allowed {
        tracing::warn!(peer = %peer.ip(), "rejected non-tailnet peer");
        return Err(ApiError::Forbidden);
    }
    Ok(next.run(req).await)
}

/// Require a valid `Authorization: Bearer <token>` header.
pub async fn require_bearer(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let presented = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match presented {
        Some(token) if state.auth.verify(token) => Ok(next.run(req).await),
        _ => Err(ApiError::Unauthorized),
    }
}
