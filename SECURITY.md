# Security Policy

WienerEnvoy runs as a **root LaunchDaemon** that listens on the network and can
**power off the machine**. That makes its security boundary unusually important,
and we take reports seriously.

## Supported versions

| Version | Supported |
|---------|-----------|
| `main`  | Yes |
| pre-1.0 tags | Best effort |

## Reporting a vulnerability

Please report privately, not in a public issue:

- Preferred: open a [GitHub Security Advisory](https://github.com/wienerlabs/wienerenvoy/security/advisories/new).
- Or email **asesenep15@gmail.com** with `WIENERENVOY SECURITY` in the subject.

We aim to acknowledge within 72 hours and to ship a fix or mitigation as fast as
the severity warrants. Please give us a reasonable window before public
disclosure.

## Scope

In scope (please report):

- Binding beyond loopback and the tailnet (any path to `0.0.0.0` or a LAN address).
- Peer-guard bypass: reaching authenticated routes from outside the tailnet.
- Bearer token handling: leakage in logs, non-constant-time comparison, weak
  generation, world-readable token files.
- Command injection in the power shellouts (`caffeinate`, `pmset`, `shutdown`).
- Privilege escalation through the root daemon (confused-deputy via the API).
- Supply chain: a malicious or vulnerable dependency surfaced by `cargo audit` or
  `cargo deny`.

Out of scope:

- Tailscale itself, macOS, and the network stack below us.
- Social engineering and physical access to an unlocked machine.
- Denial of service that requires already being authenticated on the tailnet.

## Hardening notes

- The daemon binds only `127.0.0.1` and the Tailscale CGNAT range (`100.64.0.0/10`).
- Every privileged route requires a constant-time bearer check; `shutdown`
  requires explicit confirmation.
- An `allowed_actions` allowlist lets operators ship metrics-only or no-shutdown
  deployments without code changes.
