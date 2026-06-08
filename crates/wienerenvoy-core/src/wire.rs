//! Request and response shapes that cross the HTTP/WebSocket boundary. All
//! serialize camelCase to match the TypeScript dashboard.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::state::{RecoveryPath, ServerState};
use crate::telemetry::{PresenceState, SystemSnapshot};

/// Frames pushed to the dashboard over the WebSocket. Internally tagged by
/// `type` so the client can switch on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsFrame {
    #[serde(rename_all = "camelCase")]
    Hello {
        version: String,
        sample_interval_ms: u64,
    },
    Metrics(SystemSnapshot),
    StateChanged(ServerState),
    PresenceChanged(PresenceState),
}

/// Response to an accepted power action, including how to recover the machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionAccepted {
    pub accepted: bool,
    pub action: String,
    pub recoverable_via: Vec<RecoveryPath>,
    pub warning: Option<String>,
}

/// Request body for a confirmable machine action.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedAction {
    #[serde(default)]
    pub confirm: bool,
    #[serde(default)]
    pub grace_secs: u64,
}

/// Request body for the keep-awake toggle.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAwakeRequest {
    pub enabled: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Request body for scheduling a wake. Exactly one of the fields should be set;
/// `relative_secs` wins if both are present.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeRequest {
    #[serde(default)]
    pub relative_secs: Option<u64>,
    #[serde(default)]
    pub at: Option<DateTime<Local>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_frame_hello_is_camel_case_tagged() {
        let frame = WsFrame::Hello {
            version: "0.1.0".to_string(),
            sample_interval_ms: 2000,
        };
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"type\":\"hello\""));
        assert!(json.contains("\"sampleIntervalMs\":2000"));
    }

    #[test]
    fn confirmed_action_defaults_to_unconfirmed() {
        let parsed: ConfirmedAction = serde_json::from_str("{}").unwrap();
        assert!(!parsed.confirm);
        assert_eq!(parsed.grace_secs, 0);
    }
}
