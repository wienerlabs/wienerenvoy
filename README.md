<div align="center">

# WienerEnvoy

**Turn your Mac mini into a home server you can fully power down at will. Tailnet-native: presence, power, and live telemetry, with no open ports.**

[![CI](https://github.com/wienerlabs/wienerenvoy/actions/workflows/ci.yml/badge.svg)](https://github.com/wienerlabs/wienerenvoy/actions/workflows/ci.yml)
[![Security](https://github.com/wienerlabs/wienerenvoy/actions/workflows/security.yml/badge.svg)](https://github.com/wienerlabs/wienerenvoy/actions/workflows/security.yml)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange)](rust-toolchain.toml)
[![Status: M1 live](https://img.shields.io/badge/status-M1%20power%20%2B%20telemetry-brightgreen)](CHANGELOG.md)

</div>

## What this is

WienerEnvoy is a control plane for a macOS home server. You pull the code onto a
Mac mini, run the installer, and it becomes a server you reach over your
[Tailscale](https://tailscale.com) network: see its telemetry, manage what it
runs, and, crucially, **shut the server function or the whole machine down
whenever you want** and bring it back.

It fills a gap the popular Linux self-hosting stacks (CasaOS, Umbrel, Coolify,
Tipi) do not: it is macOS-native, written in Rust, tailnet-first, and treats real
power control (sleep, restart, shutdown, scheduled wake) as a first-class feature
rather than an afterthought.

Two design decisions shape everything:

- **No open ports.** The daemon binds only to loopback and your Tailscale address
  (`100.64.0.0/10`), never `0.0.0.0`. Reaching it means being on your tailnet.
- **Granular shutdown.** Three levels: a single service, the whole server
  function ("Envoy off", daemon stays reachable), or the machine itself (sleep,
  restart, shutdown), with the recovery path shown before you act.

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│  Dashboard      Next.js static export (Overview / Power / Sys)  │
│                 your MacBook browser or phone                   │
└───────────────────────────────┬───────────────────────────────┘
                                 │  HTTP + WebSocket over Tailscale
                                 │  bearer auth, tailnet peer-guard
┌───────────────────────────────▼───────────────────────────────┐
│  wienerenvoy-daemon (Rust)     Axum  ·  loopback + 100.64/10    │
│  root LaunchDaemon             :4747                            │
└───────────────────────────────┬───────────────────────────────┘
                                 │  core traits (PowerController, KeepAwake)
┌───────────────────────────────▼───────────────────────────────┐
│  wienerenvoy-platform-macos    caffeinate · pmset · shutdown    │
│                                sysinfo telemetry                │
└───────────────────────────────────────────────────────────────┘

Read-only, alongside (never embedded): tailscaled LocalAPI / `tailscale status`.
```

## Status

**M1 shipped.** The daemon controls real power (keep-awake via `caffeinate`;
sleep, restart, and shutdown with a sleep-default / shutdown-confirm split;
scheduled wake via `pmset`) and streams live `sysinfo` telemetry over a
WebSocket. It auto-resolves and binds the tailnet address. Both the dashboard
and the `wenvoy` CLI drive it. M2 (container orchestration) is next. See
[CHANGELOG.md](CHANGELOG.md) and the roadmap below.

## Quickstart (development)

```bash
git clone https://github.com/wienerlabs/wienerenvoy
cd wienerenvoy
./scripts/dev-bootstrap.sh

# terminal 1: the daemon (dev token in /tmp)
WIENERENVOY_AUTH__TOKEN_PATH=/tmp/wenvoy-token cargo run -p wienerenvoy-daemon

# terminal 2: the dashboard
WIENERENVOY_DAEMON_URL=http://127.0.0.1:4747 pnpm -C web dev
```

Then open http://localhost:3000. The `wenvoy` CLI drives the same daemon:

```bash
wenvoy status                 # daemon and server state
wenvoy metrics                # live CPU / memory / disk / network
wenvoy keep-awake on          # hold the machine awake
wenvoy power sleep            # sleep (recoverable over the tailnet)
wenvoy power shutdown --yes   # shut down (Wake-on-LAN recovery only)
```

## Installing on the Mac mini

```bash
sudo ./install/install.sh
sudo wenvoy token show     # paste this into the dashboard once
```

The installer places the daemon as a root LaunchDaemon (so power actions need no
`sudo`), stores config and token under `/Library/Application Support/WienerEnvoy`,
enables Wake-on-LAN, and starts the service. Uninstall with
`sudo ./install/uninstall.sh`.

## Roadmap

| Milestone | Scope |
|-----------|-------|
| M0 (done) | Bootstrap: workspace, brand, governance, CI, daemon hello |
| M1 (done) | Power and presence: keep-awake, sleep/restart/shutdown, live telemetry |
| M2 | Container orchestration (Docker stacks, app catalog, service-level control) |
| M3 | Deep Tailscale: zero-password whois auth, node list, Serve/Funnel |
| M4 | Files and storage (browser, SMB/WebDAV/Taildrive, Time Machine) |
| M5 | Media services catalog |
| M6 | Backup, DNS, observability, alerts |
| M7 | Hardening, signing/notarization, auto-update, v1.0.0 |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md). The short version:

```bash
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm -C web typecheck && pnpm -C web build
```

## License

[AGPL-3.0-or-later](LICENSE). WienerEnvoy is a network-facing, self-hosted
service: the AGPL ensures that anyone who runs a modified version and exposes it
over a network shares their changes back.

Built by [Wiener Labs](https://github.com/wienerlabs).
