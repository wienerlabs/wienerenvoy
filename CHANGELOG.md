# Changelog

All notable changes to WienerEnvoy are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (M2: container orchestration)

- **Docker integration.** New `wienerenvoy-docker` crate (bollard) connects to the
  local Docker (Docker Desktop, OrbStack, or Colima; socket auto-resolved), lists
  containers grouped into Compose stacks by the `com.docker.compose.project`
  label, and controls them. The app catalog lands in M2.2.
- **Services API.** `GET /api/v1/services` (stacks + standalone; Docker-absent
  graceful), `POST /api/v1/services/control` (stack or container start/stop/
  restart), and `GET /api/v1/services/logs` (container log tail). All gated by the
  `services` allowlist entry.
- **Services dashboard.** Services page listing stacks and containers with status
  pills, per-stack start/stop/restart buttons, and a per-container log viewer.
- **`wenvoy` CLI.** `services` (list), `service start|stop|restart <id> [--stack]`,
  and `service logs <id>`.

### Added (M1: live power and telemetry)

- **Power control.** Keep-awake via a managed `caffeinate` child (orphans
  reconciled via pid-file on restart, guarded against PID reuse); sleep, restart,
  and shutdown with the sleep-default and shutdown-confirm asymmetry; scheduled
  wake via `pmset`; server-level on/off. Actions are checked against the config
  allowlist and recover-path metadata is returned to the caller.
- **Live telemetry.** `sysinfo` CPU, memory, disk, and network sampled on an
  interval and streamed over a WebSocket (token via query param), with a cached
  snapshot for one-shot reads and `GET /api/v1/system/info`.
- **Dashboard.** Live Overview, System, and Power pages driven by a reconnecting
  WebSocket, sparklines, a keep-awake toggle, confirm-gated power actions,
  scheduled wake, a token gate, and trailing-slash routing for deep links.
- **Tests.** Recording power/keep-awake mocks plus mock-backed HTTP integration
  tests (auth, confirmation, action acceptance).
- **`wenvoy` CLI.** `status`, `metrics`, `info`, `keep-awake on|off`,
  `server on|off`, and `power sleep|restart|shutdown` (restart and shutdown
  require `--yes`), all over the authenticated daemon API.
- **Tailnet auto-bind.** The daemon resolves its Tailscale address via the
  `tailscale` CLI and binds it automatically; loopback only when Tailscale is
  absent.

### Added (M0: bootstrap)

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
