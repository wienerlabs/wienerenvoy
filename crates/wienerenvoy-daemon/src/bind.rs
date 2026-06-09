//! Bind-address resolution. The whole security story starts here: `0.0.0.0` is
//! never a code path. The daemon binds loopback, the auto-resolved tailnet
//! address (from the Tailscale CLI), and any explicit `extra_bind` addresses.
//! An unspecified address is refused even if configured.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Resolve the set of socket addresses to bind. `tailnet_ip` is the
/// auto-resolved Tailscale address (see `wienerenvoy_core::tailscale`), or
/// `None` when Tailscale is absent. Always safe-by-default: if nothing
/// resolves, falls back to loopback only.
#[must_use]
pub fn resolve_binds(
    port: u16,
    allow_loopback: bool,
    extra: &[String],
    tailnet_ip: Option<Ipv4Addr>,
) -> Vec<SocketAddr> {
    let mut addrs: Vec<SocketAddr> = Vec::new();

    if allow_loopback {
        addrs.push(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
    }

    if let Some(ip) = tailnet_ip {
        tracing::info!(%ip, "binding resolved tailnet address");
        addrs.push(SocketAddr::new(IpAddr::V4(ip), port));
    }

    for entry in extra {
        match entry.parse::<IpAddr>() {
            Ok(ip) if ip.is_unspecified() => {
                tracing::warn!(addr = %entry, "refusing to bind an unspecified address; skipped");
            }
            Ok(ip) => addrs.push(SocketAddr::new(ip, port)),
            Err(_) => tracing::warn!(addr = %entry, "invalid extra_bind address; skipped"),
        }
    }

    if addrs.is_empty() {
        tracing::warn!("no bind addresses resolved; binding loopback as a safe default");
        addrs.push(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
    }

    addrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_only_by_default() {
        let b = resolve_binds(4747, true, &[], None);
        assert_eq!(b.len(), 1);
        assert!(b[0].ip().is_loopback());
    }

    #[test]
    fn refuses_unspecified() {
        let b = resolve_binds(4747, true, &["0.0.0.0".to_string()], None);
        assert!(b.iter().all(|a| !a.ip().is_unspecified()));
    }

    #[test]
    fn accepts_explicit_tailnet_ip() {
        let b = resolve_binds(4747, true, &["100.64.0.5".to_string()], None);
        assert!(
            b.iter()
                .any(|a| a.ip() == IpAddr::V4(Ipv4Addr::new(100, 64, 0, 5)))
        );
    }

    #[test]
    fn includes_resolved_tailnet_ip() {
        let tailnet = Ipv4Addr::new(100, 100, 0, 9);
        let b = resolve_binds(4747, true, &[], Some(tailnet));
        assert!(b.iter().any(|a| a.ip() == IpAddr::V4(tailnet)));
        assert!(b.iter().any(|a| a.ip().is_loopback()));
    }

    #[test]
    fn never_empty() {
        let b = resolve_binds(4747, false, &["bogus".to_string()], None);
        assert!(!b.is_empty());
    }
}
