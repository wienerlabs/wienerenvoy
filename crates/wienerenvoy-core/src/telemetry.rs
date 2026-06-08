//! Telemetry wire types. All are serialized camelCase to stay idiomatic for
//! the TypeScript dashboard that consumes them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::presence::Presence;

/// One sampled snapshot of machine telemetry, broadcast over the WebSocket and
/// returned by `GET /api/v1/system/metrics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub captured_at: DateTime<Utc>,
    pub presence: Presence,
    pub uptime_secs: u64,
    pub cpu: CpuUsage,
    pub memory: MemoryUsage,
    pub disks: Vec<DiskUsage>,
    pub network: NetworkRate,
    /// 1, 5, and 15 minute load averages.
    pub load_avg: [f64; 3],
    /// Best-effort; often empty on Apple Silicon where SMC sensors are not
    /// exposed to userspace sampling.
    pub temperatures: Vec<TempSensor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuUsage {
    pub usage_pct: f32,
    pub cores: usize,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    pub mount: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkRate {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempSensor {
    pub label: String,
    pub celsius: f32,
}

/// Static-ish machine info returned by `GET /api/v1/system/info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub hostname: String,
    pub os_version: String,
    pub cpu_model: String,
    pub mem_total: u64,
    pub boot_time: DateTime<Utc>,
    pub tailnet_ip: Option<String>,
    pub daemon_version: String,
}

/// Keep-awake / presence detail returned by `GET /api/v1/presence`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceState {
    pub keep_awake: bool,
    pub backend: String,
    pub holder_pid: Option<u32>,
    pub since: DateTime<Utc>,
}
