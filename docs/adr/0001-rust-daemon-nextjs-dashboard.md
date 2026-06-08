# ADR-0001: Rust daemon plus Next.js dashboard

- Status: Accepted
- Date: 2026-06-08
- Deciders: @kh0ra

## Context

WienerEnvoy needs a long-lived system service on the Mac mini (power control,
telemetry, launchd integration) and a control surface a person uses from a
laptop or phone. These have different requirements: the service wants low
footprint, a single binary, strong safety, and easy launchd integration; the UI
wants fast iteration and a large contributor pool.

## Decision

Split the system into a **Rust daemon** (Tokio + Axum, HTTP and WebSocket) and a
**Next.js dashboard** (static export served by the daemon or in dev on its own
port). They communicate over an authenticated, tailnet-only API.

## Consequences

- The daemon is a single binary with a small runtime, `unsafe`-forbidden, and a
  clean launchd story. Power is behind traits so it is testable without touching
  the real machine.
- The dashboard uses the stack the team already knows, lowering the bar for UI
  contributions, and ships as a static export with no server runtime to operate.
- Two languages in one repo means two toolchains and a JSON contract maintained
  by hand (revisited if it gets painful; `ts-rs` is a future option).

## Alternatives considered

- **Full Rust with an embedded web UI (Leptos/Dioxus)**: one language and one
  binary, but a much smaller frontend contributor pool and higher UI iteration
  cost. Rejected for a project that wants outside contributions.
- **Go daemon**: faster to write, but Rust gives stronger guarantees for a root
  service that can power off the machine, and matches the team's other system
  projects.
