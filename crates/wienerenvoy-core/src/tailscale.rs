//! Tailscale awareness. WienerEnvoy does not embed Tailscale (the Rust port is
//! experimental and `libtailscale` drags in the Go runtime). Instead the daemon
//! reads tailnet state read-only: the assigned `100.64.0.0/10` address to bind
//! to, and node status. M1 ships only address classification and a status-shape
//! placeholder; the LocalAPI/CLI reader lands with M3.

use std::net::Ipv4Addr;

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
}
