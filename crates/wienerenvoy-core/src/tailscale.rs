//! Tailscale awareness. WienerEnvoy does not embed Tailscale (the Rust port is
//! experimental and `libtailscale` drags in the Go runtime). Instead the daemon
//! reads tailnet state read-only: M1 resolves the bind address via the
//! `tailscale` CLI (`tailscale ip -4`); the richer LocalAPI status reader lands
//! with M3.

use std::net::Ipv4Addr;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Whether an IPv4 address is in the Tailscale CGNAT range, 100.64.0.0/10
/// (RFC 6598). The daemon binds only to loopback and an address in this range,
/// never to 0.0.0.0.
#[must_use]
pub fn is_cgnat(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    // First octet 100, second octet 64..=127 (top two bits 01).
    o[0] == 100 && (o[1] & 0xC0) == 0x40
}

/// Resolve the machine's Tailscale IPv4 address by shelling out to the
/// `tailscale` CLI. Returns the first address in the CGNAT range, or `None` if
/// Tailscale is not installed or not up (in which case the daemon binds
/// loopback only). Read-only: this never changes tailnet state.
#[must_use]
pub fn resolve_tailnet_ipv4(cli: &str) -> Option<Ipv4Addr> {
    let program = tailscale_program(cli);
    let output = Command::new(program).args(["ip", "-4"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse::<Ipv4Addr>().ok())
        .find(|ip| is_cgnat(*ip))
}

/// Resolve the `tailscale` CLI path. "auto" probes the standalone install
/// location, then falls back to a PATH lookup.
fn tailscale_program(cli: &str) -> String {
    if cli != "auto" {
        return cli.to_string();
    }
    const STANDALONE: &str = "/usr/local/bin/tailscale";
    if std::path::Path::new(STANDALONE).exists() {
        STANDALONE.to_string()
    } else {
        "tailscale".to_string()
    }
}

/// A minimal view of `tailscale status --json` `Self`. Expanded in M3.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TailnetSelf {
    #[serde(default)]
    pub tailscale_ips: Vec<String>,
    #[serde(default)]
    pub dns_name: String,
    #[serde(default)]
    pub online: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgnat_range_classified() {
        assert!(is_cgnat(Ipv4Addr::new(100, 64, 0, 1)));
        assert!(is_cgnat(Ipv4Addr::new(100, 100, 100, 100)));
        assert!(is_cgnat(Ipv4Addr::new(100, 127, 255, 255)));
    }

    #[test]
    fn non_cgnat_rejected() {
        assert!(!is_cgnat(Ipv4Addr::new(100, 63, 0, 1)));
        assert!(!is_cgnat(Ipv4Addr::new(100, 128, 0, 1)));
        assert!(!is_cgnat(Ipv4Addr::new(192, 168, 1, 1)));
        assert!(!is_cgnat(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!is_cgnat(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn explicit_cli_path_is_used_verbatim() {
        // A bogus explicit path simply fails to run and yields None.
        assert!(resolve_tailnet_ipv4("/nonexistent/tailscale").is_none());
    }
}
