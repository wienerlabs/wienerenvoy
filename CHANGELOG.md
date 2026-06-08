# Changelog

All notable changes to WienerEnvoy are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **M0 bootstrap.** Cargo workspace with four crates (`wienerenvoy-core`,
  `wienerenvoy-platform-macos`, `wienerenvoy-daemon`, `wienerenvoy-cli`), the
  Next.js dashboard brand layer, full governance, and CI.
- **Daemon.** Axum HTTP server binding only loopback and the Tailscale CGNAT
  range (never `0.0.0.0`), a peer-guard rejecting non-tailnet callers,
  constant-time bearer auth, an unauthenticated `/health`, a real
  `/api/v1/state`, typed `501` stubs for the M1 power and telemetry routes, a
  static dashboard fallback, and graceful shutdown on SIGINT and SIGTERM.
- **Granular shutdown state machine** in `wienerenvoy-core`: server-level
  `Active` and `ServerOff`, machine commands (`sleep`, `restart`, `shutdown`),
  with the sleep-default and shutdown-confirm asymmetry and recovery paths.
- **`wenvoy` CLI.** `status`, `token show`, install/uninstall guidance, and shell
  completions, with the WienerLabs terminal brand layer.
- **Config and auth.** Figment-layered config and a bearer-token store with
  CSPRNG generation, `secrecy` wrapping, and permission self-checks.
- **macOS integration.** Root LaunchDaemon plist and idempotent install scripts.
- **Dashboard.** Brand layer (Funnel Display, ink/cream palette, online/standby/
  offline status pills, dark and light themes) and the Overview, Power, and
  System pages as M1-ready shells.

### Notes

- Power and telemetry actions are stubbed in M0 and return zeroed or `501`
  responses; they are wired to real `caffeinate`/`pmset`/`shutdown` and `sysinfo`
  sampling in M1.
