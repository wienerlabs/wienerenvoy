//! Service wire types: Docker containers grouped into Compose stacks. The
//! daemon's `wienerenvoy-docker` crate produces these; the dashboard and CLI
//! consume them. Service runtime state is authoritative in Docker itself, so
//! there is no separate state machine here.

use serde::{Deserialize, Serialize};

/// Whether Docker is reachable, and its version if so.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerStatus {
    pub available: bool,
    pub version: Option<String>,
}

/// One container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInfo {
    /// Short (12-char) container id.
    pub id: String,
    pub name: String,
    pub image: String,
    /// Docker state: `running`, `exited`, `paused`, etc.
    pub state: String,
    /// Human status line, e.g. `Up 2 hours`.
    pub status: String,
}

/// A Compose stack: containers sharing a `com.docker.compose.project` label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackSummary {
    pub name: String,
    pub running: usize,
    pub total: usize,
    pub containers: Vec<ContainerInfo>,
}

/// The full services view returned by `GET /api/v1/services`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesView {
    pub docker: DockerStatus,
    /// Compose stacks, sorted by name.
    pub stacks: Vec<StackSummary>,
    /// Containers not part of any Compose project.
    pub standalone: Vec<ContainerInfo>,
}
