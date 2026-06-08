//! System telemetry sampler.
//!
//! M0: returns a zeroed snapshot so the daemon and dashboard wire end to end.
//! M1 wires `sysinfo` with the CPU warm-up refresh (the first sample is invalid
//! without it) and best-effort temperatures (usually empty on Apple Silicon).

use chrono::Utc;
use wienerenvoy_core::presence::Presence;
use wienerenvoy_core::telemetry::{
    CpuUsage, DiskUsage, MemoryUsage, NetworkRate, SystemSnapshot, TempSensor,
};

/// Samples machine telemetry on demand.
#[derive(Debug, Default)]
pub struct MetricsSampler {
    _private: (),
}

impl MetricsSampler {
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Produce a telemetry snapshot. M0 returns zeros; M1 reads real values.
    pub fn sample(&mut self) -> SystemSnapshot {
        SystemSnapshot {
            captured_at: Utc::now(),
            presence: Presence::Online,
            uptime_secs: 0,
            cpu: CpuUsage {
                usage_pct: 0.0,
                cores: 0,
                per_core: Vec::new(),
            },
            memory: MemoryUsage {
                used_bytes: 0,
                total_bytes: 0,
                available_bytes: 0,
                swap_used_bytes: 0,
                swap_total_bytes: 0,
            },
            disks: Vec::<DiskUsage>::new(),
            network: NetworkRate {
                rx_bytes_per_sec: 0,
                tx_bytes_per_sec: 0,
            },
            load_avg: [0.0, 0.0, 0.0],
            temperatures: Vec::<TempSensor>::new(),
        }
    }
}
