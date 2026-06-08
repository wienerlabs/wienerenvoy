//! High-level presence of the home server, as shown by the dashboard pill.

use serde::{Deserialize, Serialize};

/// Coarse presence state surfaced to the dashboard status pill.
///
/// This maps to the brand status colors: `Online` is the signal green,
/// `Standby` the amber, `Offline` the rose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    /// Server function active, machine awake and reachable.
    Online,
    /// Server function paused ("Envoy off") or machine idle but reachable.
    Standby,
    /// Machine asleep, shut down, or unreachable.
    Offline,
}

impl Presence {
    /// Stable string form used in logs and the wire protocol.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Standby => "standby",
            Self::Offline => "offline",
        }
    }
}

impl std::fmt::Display for Presence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
