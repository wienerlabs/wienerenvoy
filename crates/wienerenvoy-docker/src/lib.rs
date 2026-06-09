//! Docker integration for WienerEnvoy.
//!
//! M2.0 ships read-only discovery: connect to the local Docker (Docker Desktop,
//! OrbStack, or Colima; socket resolved automatically or from `DOCKER_HOST`),
//! and list containers grouped into Compose stacks by the
//! `com.docker.compose.project` label. Stack management (`docker compose
//! up/down`) and log streaming land in M2.1; the app catalog in M2.2.
//!
//! Docker absence is never an error: the handle reports `available: false` and
//! returns an empty services view, so the daemon and dashboard degrade
//! gracefully on a machine without Docker.

pub mod client;
pub mod error;

pub use client::DockerHandle;
pub use error::DockerError;
