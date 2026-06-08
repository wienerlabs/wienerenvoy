//! HTTP route handlers. M0 ships `/health`, the auth probe, and a real
//! `/api/v1/state`; the system, power, presence, and WebSocket routes are typed
//! `501` stubs until M1 fills them in.

pub mod health;
pub mod server_state;
pub mod stub;
