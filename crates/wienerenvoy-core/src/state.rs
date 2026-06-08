//! The granular shutdown state machine.
//!
//! Three control scopes are modeled:
//!   - ServiceLevel: a single managed service (Docker in M2; not represented
//!     here yet, the `services` map lands with that milestone).
//!   - ServerLevel: the "Envoy off" toggle. `Active` runs managed presence;
//!     `ServerOff` releases it while the daemon stays up and reachable.
//!   - MachineLevel: `Sleep` / `Restart` / `Shutdown`. These are commands, not
//!     resting states, so they live in `MachineCommand`, not `ServerLevel`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::power::MachineCommand;
use crate::presence::Presence;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    #[error("action `{0}` requires explicit confirmation")]
    ConfirmationRequired(String),
}

/// The resting state of the server function (distinct from machine power).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerLevel {
    /// Managed presence engaged, all functions available.
    Active,
    /// "Envoy off": presence released, daemon still running and reachable so it
    /// can be turned back on remotely.
    ServerOff,
}

/// How a machine can be brought back after a power action. Surfaced to the
/// dashboard so the user always sees the recovery path before acting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPath {
    /// Wake over the tailnet (requires network wake configured).
    TailnetWake,
    /// Wake-on-LAN magic packet from a same-segment device.
    Wol,
    /// A pre-scheduled `pmset` wake will fire.
    ScheduledWake,
    /// The daemon returns on its own via launchd `RunAtLoad` after boot.
    LaunchdRelaunch,
    /// Only Wake-on-LAN can recover this; the tailnet is gone.
    WolOnly,
}

/// A point-in-time view of the server state, returned by `GET /api/v1/state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerState {
    pub level: ServerLevel,
    pub presence: Presence,
    pub since: DateTime<Utc>,
    pub recoverable_via: Vec<RecoveryPath>,
}

/// Single-writer state machine for the server-level scope. The daemon guards
/// this behind a mutex and publishes a WebSocket frame on every mutation.
#[derive(Debug, Clone)]
pub struct ServerStateMachine {
    level: ServerLevel,
    since: DateTime<Utc>,
}

impl ServerStateMachine {
    #[must_use]
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            level: ServerLevel::Active,
            since: now,
        }
    }

    #[must_use]
    pub fn level(&self) -> ServerLevel {
        self.level
    }

    /// Apply a server-level transition. `Active` and `ServerOff` are freely
    /// reachable from each other; both keep the daemon alive.
    pub fn apply_level(&mut self, target: ServerLevel, now: DateTime<Utc>) -> ServerState {
        if self.level != target {
            self.level = target;
            self.since = now;
        }
        self.snapshot()
    }

    /// Validate that a machine command may be issued given the confirm flag.
    pub fn can_issue(&self, cmd: MachineCommand, confirmed: bool) -> Result<(), StateError> {
        if cmd.requires_confirm() && !confirmed {
            return Err(StateError::ConfirmationRequired(cmd.as_str().to_string()));
        }
        Ok(())
    }

    /// Recovery paths for a given machine command, independent of current state.
    #[must_use]
    pub fn recovery_for(cmd: MachineCommand) -> Vec<RecoveryPath> {
        match cmd {
            MachineCommand::Sleep => vec![
                RecoveryPath::TailnetWake,
                RecoveryPath::Wol,
                RecoveryPath::ScheduledWake,
            ],
            MachineCommand::Restart => vec![RecoveryPath::LaunchdRelaunch],
            MachineCommand::Shutdown => vec![RecoveryPath::WolOnly],
        }
    }

    /// Build the public state view for the current level.
    #[must_use]
    pub fn snapshot(&self) -> ServerState {
        let presence = match self.level {
            ServerLevel::Active => Presence::Online,
            ServerLevel::ServerOff => Presence::Standby,
        };
        ServerState {
            level: self.level,
            presence,
            since: self.since,
            recoverable_via: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap()
    }

    #[test]
    fn starts_active_online() {
        let sm = ServerStateMachine::new(t0());
        let snap = sm.snapshot();
        assert_eq!(snap.level, ServerLevel::Active);
        assert_eq!(snap.presence, Presence::Online);
    }

    #[test]
    fn server_off_is_standby_and_reversible() {
        let mut sm = ServerStateMachine::new(t0());
        let off = sm.apply_level(ServerLevel::ServerOff, t0());
        assert_eq!(off.level, ServerLevel::ServerOff);
        assert_eq!(off.presence, Presence::Standby);
        let on = sm.apply_level(ServerLevel::Active, t0());
        assert_eq!(on.level, ServerLevel::Active);
        assert_eq!(on.presence, Presence::Online);
    }

    #[test]
    fn sleep_needs_no_confirm() {
        let sm = ServerStateMachine::new(t0());
        assert!(sm.can_issue(MachineCommand::Sleep, false).is_ok());
    }

    #[test]
    fn shutdown_requires_confirm() {
        let sm = ServerStateMachine::new(t0());
        assert_eq!(
            sm.can_issue(MachineCommand::Shutdown, false),
            Err(StateError::ConfirmationRequired("shutdown".to_string()))
        );
        assert!(sm.can_issue(MachineCommand::Shutdown, true).is_ok());
    }

    #[test]
    fn restart_requires_confirm() {
        let sm = ServerStateMachine::new(t0());
        assert!(sm.can_issue(MachineCommand::Restart, false).is_err());
        assert!(sm.can_issue(MachineCommand::Restart, true).is_ok());
    }

    #[test]
    fn shutdown_recovery_is_wol_only() {
        assert_eq!(
            ServerStateMachine::recovery_for(MachineCommand::Shutdown),
            vec![RecoveryPath::WolOnly]
        );
    }

    #[test]
    fn sleep_recovery_includes_tailnet() {
        let paths = ServerStateMachine::recovery_for(MachineCommand::Sleep);
        assert!(paths.contains(&RecoveryPath::TailnetWake));
    }
}
