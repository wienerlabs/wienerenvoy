//! Daemon configuration. Layered via figment: built-in defaults, then a TOML
//! file, then `WIENERENVOY_*` environment overrides.

use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to load config: {0}")]
    Load(#[from] figment::Error),
}

/// Top-level daemon configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub power: PowerConfig,
    #[serde(default)]
    pub tailscale: TailscaleConfig,
    #[serde(default)]
    pub permissions: PermissionsConfig,
}

impl Config {
    /// Load defaults, overlay the TOML file at `path` if it exists, then apply
    /// `WIENERENVOY_*` env overrides.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let cfg = Figment::from(figment::providers::Serialized::defaults(Config::default()))
            .merge(Toml::file(path))
            .merge(Env::prefixed("WIENERENVOY_").split("__"))
            .extract()?;
        Ok(cfg)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// TCP port the daemon listens on (loopback plus the tailnet interface).
    pub http_port: u16,
    /// Extra explicit IPs to bind in addition to loopback and the tailnet IP.
    pub extra_bind: Vec<String>,
    /// Whether to bind loopback (127.0.0.1) for the local CLI.
    pub allow_loopback: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 4747,
            extra_bind: Vec::new(),
            allow_loopback: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Where the bearer token is stored.
    pub token_path: PathBuf,
    /// Storage backend: "file" (default) or "keyring".
    pub backend: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_path: PathBuf::from("/Library/Application Support/WienerEnvoy/token"),
            backend: "file".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Telemetry sampling interval in milliseconds.
    pub sample_interval_ms: u64,
    /// Whether to publish per-core CPU usage (larger frames).
    pub publish_per_core: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            sample_interval_ms: 2000,
            publish_per_core: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Recommended machine action ("sleep"); enforces the recoverable default.
    pub default_machine_action: String,
    pub allow_shutdown: bool,
    pub allow_restart: bool,
    pub shutdown_requires_confirm: bool,
    /// `caffeinate` flags for the keep-awake assertion.
    pub keep_awake_flags: String,
    /// Enable Wake-on-LAN (`pmset womp 1`) during install.
    pub enable_wol_on_install: bool,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            default_machine_action: "sleep".to_string(),
            allow_shutdown: true,
            allow_restart: true,
            shutdown_requires_confirm: true,
            keep_awake_flags: "-s".to_string(),
            enable_wol_on_install: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailscaleConfig {
    /// Path to the tailscaled LocalAPI unix socket.
    pub localapi_socket: PathBuf,
    /// Path to the `tailscale` CLI, or "auto" to probe known locations.
    pub status_cli: String,
}

impl Default for TailscaleConfig {
    fn default() -> Self {
        Self {
            localapi_socket: PathBuf::from("/var/run/tailscaled.socket"),
            status_cli: "auto".to_string(),
        }
    }
}

/// Coarse allowlist so an operator can ship a metrics-only or no-shutdown
/// deployment without code changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub allowed_actions: Vec<String>,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            allowed_actions: vec![
                "metrics".to_string(),
                "presence".to_string(),
                "sleep".to_string(),
                "restart".to_string(),
                "shutdown".to_string(),
                "server_off".to_string(),
                "services".to_string(),
            ],
        }
    }
}

impl PermissionsConfig {
    /// Whether a named action is permitted by this deployment.
    #[must_use]
    pub fn allows(&self, action: &str) -> bool {
        self.allowed_actions.iter().any(|a| a == action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.server.http_port, 4747);
        assert!(c.server.allow_loopback);
        assert_eq!(c.power.default_machine_action, "sleep");
        assert!(c.power.shutdown_requires_confirm);
    }

    #[test]
    fn permission_allowlist_checks() {
        let p = PermissionsConfig::default();
        assert!(p.allows("sleep"));
        assert!(!p.allows("format_disk"));
    }

    #[test]
    fn missing_file_falls_back_to_defaults() {
        let c = Config::load(Path::new("/nonexistent/wienerenvoy.toml")).unwrap();
        assert_eq!(c.server.http_port, 4747);
    }
}
