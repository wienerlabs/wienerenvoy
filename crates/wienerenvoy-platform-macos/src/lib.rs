//! macOS platform layer for WienerEnvoy.
//!
//! Implements the `wienerenvoy-core` control traits against macOS facilities:
//! `caffeinate` for keep-awake, `pmset`/`shutdown` for power, and `sysinfo` for
//! telemetry.
//!
//! M0 ships compiling stubs that do not touch the machine (so the daemon wires
//! end to end and the dashboard renders). M1 fills in the real shellouts with
//! the caffeinate PID-reconciliation, the sleep-default / shutdown-confirm
//! asymmetry, and live sysinfo sampling.

pub mod keepawake;
pub mod metrics;
pub mod power;

pub use keepawake::CaffeinateKeeper;
pub use metrics::{MetricsSampler, system_info};
pub use power::MacOsPower;
