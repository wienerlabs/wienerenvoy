//! HTTP route handlers. M1 wires the real system, presence, power, and
//! WebSocket routes; `/api/v1/services` stays a typed `501` stub until M2.

pub mod health;
pub mod power;
pub mod presence;
pub mod server_state;
pub mod stub;
pub mod system;
pub mod ws;
