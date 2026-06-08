//! Shared daemon application state, cloned into every handler.

use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};
use wienerenvoy_core::{
    AuthStore, Config, KeepAwake, PowerController, ServerStateMachine, SystemSnapshot, WsFrame,
};

/// Application state shared across handlers. Cheap to clone (all `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthStore>,
    pub state: Arc<Mutex<ServerStateMachine>>,
    pub power: Arc<dyn PowerController>,
    pub keepawake: Arc<dyn KeepAwake>,
    /// Latest telemetry sample, refreshed by the sampler task.
    pub snapshot: Arc<Mutex<Option<SystemSnapshot>>>,
    /// Broadcast channel for WebSocket frames (metrics, state, presence).
    pub events: broadcast::Sender<WsFrame>,
    pub version: String,
}
