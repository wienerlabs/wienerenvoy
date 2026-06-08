//! Shared daemon application state, cloned into every handler.

use std::sync::Arc;

use tokio::sync::Mutex;
use wienerenvoy_core::{AuthStore, Config, KeepAwake, PowerController, ServerStateMachine};

/// Application state shared across handlers. Cheap to clone (all `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<AuthStore>,
    pub state: Arc<Mutex<ServerStateMachine>>,
    /// Machine power control. Wired now; exercised by power routes in M1.
    pub power: Arc<dyn PowerController>,
    /// Keep-awake / presence control. Wired now; exercised in M1.
    pub keepawake: Arc<dyn KeepAwake>,
    pub version: String,
}
