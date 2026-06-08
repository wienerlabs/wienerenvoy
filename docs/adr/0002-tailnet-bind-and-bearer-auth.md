# ADR-0002: Tailnet-only bind, bearer auth, and no embedded Tailscale

- Status: Accepted
- Date: 2026-06-08
- Deciders: @kh0ra

## Context

The daemon runs as root and can power off the machine, so its network exposure
is the entire security story. We also need to know the machine's tailnet address
and status. One option is to embed Tailscale in the process; another is to run
Tailscale alongside and read it.

## Decision

- **Bind only loopback and the Tailscale CGNAT range** (`100.64.0.0/10`). Never
  `0.0.0.0`. A peer-guard middleware rejects any connection from outside that set
  as defense in depth on top of the bind restriction.
- **Authenticate every privileged route with a bearer token**, generated from the
  OS CSPRNG, wrapped in `secrecy`, stored with restrictive permissions, and
  compared in constant time. `shutdown` additionally requires explicit
  confirmation.
- **Do not embed Tailscale.** Run `tailscaled` as a separate system service and
  read tailnet state read-only (LocalAPI socket, or `tailscale status --json`).

## Consequences

- The daemon is unreachable from the LAN or the internet without being on the
  tailnet, which is the property we want for a machine-power API.
- No Go runtime is dragged into the process, and there is no dependency on
  unaudited cryptography.
- The bearer token is the M1 mechanism. Zero-password tailnet auth via LocalAPI
  `whois` is a roadmap item (M3); callers go through an `Authenticator` trait so
  the swap is local.

## Alternatives considered

- **Embed Tailscale** via `tailscale-rs`: still an experimental preview with
  unaudited cryptography and a required `TS_RS_EXPERIMENT` flag. Rejected for a
  root daemon.
- **Embed via `libtailscale`**: pulls the entire Go runtime into the process and
  fights the Tokio runtime for process lifecycle. Rejected.
- **Public reverse proxy** (Caddy/Cloudflare Tunnel): broadens the attack surface
  on a machine-power API. Rejected in favor of tailnet-only.
