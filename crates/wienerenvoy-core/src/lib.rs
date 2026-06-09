//! Platform-agnostic domain core for WienerEnvoy.
//!
//! This crate holds the wire types, the control traits (`PowerController`,
//! `KeepAwake`), the granular shutdown state machine, config loading, and the
//! bearer-token auth store. No platform shellouts live here; macOS specifics
//! are in `wienerenvoy-platform-macos`. The daemon and CLI depend on these
//! contracts so the platform layer stays swappable and testable.

pub mod auth;
pub mod config;
pub mod power;
pub mod presence;
pub mod service;
pub mod state;
pub mod tailscale;
pub mod telemetry;
pub mod wire;

#[cfg(feature = "testutil")]
pub mod testutil;

pub use auth::{AuthError, AuthStore};
pub use config::{Config, ConfigError};
pub use power::{KeepAwake, MachineCommand, PowerController, PowerError, PowerInfo, WakeSpec};
pub use presence::Presence;
pub use service::{
    ContainerInfo, ControlResult, DockerStatus, LogLines, ServiceAction, ServiceControl,
    ServiceTarget, ServicesView, StackSummary,
};
pub use state::{RecoveryPath, ServerLevel, ServerState, ServerStateMachine, StateError};
pub use telemetry::{
    CpuUsage, DiskUsage, MemoryUsage, NetworkRate, PresenceState, SystemInfo, SystemSnapshot,
    TempSensor,
};
pub use wire::{ActionAccepted, ConfirmedAction, KeepAwakeRequest, WakeRequest, WsFrame};

/// The crate version, surfaced over the API for client compatibility checks.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
