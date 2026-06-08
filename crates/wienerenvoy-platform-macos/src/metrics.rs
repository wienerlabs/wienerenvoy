//! System telemetry sampling via `sysinfo`.
//!
//! Notes baked in from the macOS reality: CPU needs a warm-up refresh before the
//! first reading is valid, and temperature sensors are usually empty on Apple
//! Silicon (so temperatures are best-effort, never an error).

use std::time::Instant;

use chrono::{DateTime, Utc};
use sysinfo::{Components, Disks, Networks, System};
use wienerenvoy_core::presence::Presence;
use wienerenvoy_core::telemetry::{
    CpuUsage, DiskUsage, MemoryUsage, NetworkRate, SystemInfo, SystemSnapshot, TempSensor,
};

/// Build a one-shot machine info record. `tailnet_ip` is resolved in M3.
#[must_use]
pub fn system_info(daemon_version: &str) -> SystemInfo {
    let sys = System::new_all();
    let boot_time =
        DateTime::from_timestamp(System::boot_time() as i64, 0).unwrap_or_else(Utc::now);
    SystemInfo {
        hostname: System::host_name().unwrap_or_default(),
        os_version: System::long_os_version().unwrap_or_else(|| "macOS".to_string()),
        cpu_model: sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default(),
        mem_total: sys.total_memory(),
        boot_time,
        tailnet_ip: None,
        daemon_version: daemon_version.to_string(),
    }
}

/// Samples machine telemetry on demand, tracking network deltas between calls.
pub struct MetricsSampler {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    last_net: Option<(Instant, u64, u64)>,
}

impl MetricsSampler {
    #[must_use]
    pub fn new() -> Self {
        let mut sys = System::new();
        // A valid CPU reading needs two refreshes spaced by at least
        // MINIMUM_CPU_UPDATE_INTERVAL, so warm it up here.
        sys.refresh_cpu_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_all();
        sys.refresh_memory();
        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_net: None,
        }
    }

    /// Produce a fresh telemetry snapshot.
    pub fn sample(&mut self) -> SystemSnapshot {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);

        let cpu_global = self.sys.global_cpu_usage();
        let per_core: Vec<f32> = self
            .sys
            .cpus()
            .iter()
            .map(sysinfo::Cpu::cpu_usage)
            .collect();
        let cores = per_core.len();

        let (mut rx_total, mut tx_total) = (0u64, 0u64);
        for (_name, data) in &self.networks {
            rx_total += data.total_received();
            tx_total += data.total_transmitted();
        }
        let now = Instant::now();
        let (rx_rate, tx_rate) = match self.last_net {
            Some((then, prev_rx, prev_tx)) => {
                let dt = now.duration_since(then).as_secs_f64().max(0.001);
                (
                    (rx_total.saturating_sub(prev_rx) as f64 / dt) as u64,
                    (tx_total.saturating_sub(prev_tx) as f64 / dt) as u64,
                )
            }
            None => (0, 0),
        };
        self.last_net = Some((now, rx_total, tx_total));

        let disks = self
            .disks
            .iter()
            .map(|d| DiskUsage {
                mount: d.mount_point().to_string_lossy().into_owned(),
                total_bytes: d.total_space(),
                used_bytes: d.total_space().saturating_sub(d.available_space()),
            })
            .collect();

        let temperatures = self
            .components
            .iter()
            .filter_map(|c| {
                c.temperature().map(|celsius| TempSensor {
                    label: c.label().to_string(),
                    celsius,
                })
            })
            .collect();

        let load = System::load_average();

        SystemSnapshot {
            captured_at: Utc::now(),
            presence: Presence::Online,
            uptime_secs: System::uptime(),
            cpu: CpuUsage {
                usage_pct: cpu_global,
                cores,
                per_core,
            },
            memory: MemoryUsage {
                used_bytes: self.sys.used_memory(),
                total_bytes: self.sys.total_memory(),
                available_bytes: self.sys.available_memory(),
                swap_used_bytes: self.sys.used_swap(),
                swap_total_bytes: self.sys.total_swap(),
            },
            disks,
            network: NetworkRate {
                rx_bytes_per_sec: rx_rate,
                tx_bytes_per_sec: tx_rate,
            },
            load_avg: [load.one, load.five, load.fifteen],
            temperatures,
        }
    }
}

impl Default for MetricsSampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_produces_a_plausible_snapshot() {
        let mut sampler = MetricsSampler::new();
        let snap = sampler.sample();
        assert!(snap.cpu.cores >= 1, "expected at least one core");
        assert!(snap.memory.total_bytes > 0, "expected nonzero total memory");
    }
}
