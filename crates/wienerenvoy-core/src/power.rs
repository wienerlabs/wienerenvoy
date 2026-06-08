//! Power control contracts. Every privileged power action goes through these
//! traits so the daemon depends on behavior, not on `caffeinate`/`pmset`
//! directly, and tests can assert against a recording mock without ever
//! putting the real machine to sleep.

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from power and presence operations.
#[derive(Debug, Error)]
pub enum PowerError {
    #[error("failed to spawn power command: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("power command `{command}` exited with {code:?}: {stderr}")]
    NonZeroExit {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("operation not permitted; the daemon must run as root")]
    NotPermitted,
    #[error("power command timed out")]
    Timeout,
    #[error("failed to parse power output: {0}")]
    Parse(String),
    #[error("action `{0}` is disabled by configuration")]
    Disabled(String),
}

/// A machine-level power command. These are transient actions, not stored
/// states; once issued, the daemon either survives (sleep) or is about to go
/// away (restart, shutdown).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineCommand {
    Sleep,
    Restart,
    Shutdown,
}

impl MachineCommand {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sleep => "sleep",
            Self::Restart => "restart",
            Self::Shutdown => "shutdown",
        }
    }

    /// Whether this command requires explicit confirmation. Sleep is
    /// recoverable over the tailnet so it does not; restart and shutdown can
    /// lock the operator out, so they do.
    #[must_use]
    pub fn requires_confirm(self) -> bool {
        matches!(self, Self::Restart | Self::Shutdown)
    }
}

/// When to wake the machine. macOS `pmset` expects local time, so the schedule
/// variant carries a `Local` timestamp deliberately.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum WakeSpec {
    /// Wake this many seconds from now.
    Relative { secs: u64 },
    /// Wake at a specific local time.
    Schedule { at: DateTime<Local> },
}

/// Power-source and pending-wake info parsed from the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerInfo {
    pub on_ac_power: bool,
    pub pending_wake: Option<DateTime<Local>>,
}

/// Controls machine-level power. Implemented by the platform crate.
#[async_trait]
pub trait PowerController: Send + Sync {
    /// Put the machine to sleep immediately. Recoverable.
    async fn sleep(&self) -> Result<(), PowerError>;
    /// Restart the machine after a grace period.
    async fn restart(&self, grace: Duration) -> Result<(), PowerError>;
    /// Shut the machine down after a grace period. Not remotely recoverable
    /// without Wake-on-LAN.
    async fn shutdown(&self, grace: Duration) -> Result<(), PowerError>;
    /// Schedule a future wake.
    async fn schedule_wake(&self, spec: WakeSpec) -> Result<(), PowerError>;
    /// Read current power-source state and any pending wake.
    async fn power_state(&self) -> Result<PowerInfo, PowerError>;
}

/// Holds a system-sleep assertion (the "keep awake" / presence function).
#[async_trait]
pub trait KeepAwake: Send + Sync {
    /// Engage the assertion with a human-readable reason.
    async fn engage(&self, reason: &str) -> Result<(), PowerError>;
    /// Release the assertion, letting the machine idle-sleep again.
    async fn release(&self) -> Result<(), PowerError>;
    /// Whether the assertion is currently engaged.
    fn is_engaged(&self) -> bool;
    /// PID of the holder process, if the backend uses one (e.g. caffeinate).
    fn holder_pid(&self) -> Option<u32>;
}
