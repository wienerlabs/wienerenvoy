# Contributing to WienerEnvoy

Thanks for helping build WienerEnvoy, an open, tailnet-native control plane that
turns a Mac mini into a home server you can fully power down at will. Contributions
are welcome: bug reports, features, docs, and platform ports.

## Ground rules

- **Conventional Commits 1.0.0** for every commit (enforced by a git hook).
- **No em dashes** anywhere: code, comments, docs, commit messages, UI copy.
  Use a colon, semicolon, a plain hyphen, or a paragraph break instead. This is
  enforced by the `commit-msg` and `pre-push` hooks.
- **Trunk-based**: short-lived branches off `main`, squash-merge.
- **Security first**: this is a root daemon that listens on the network and can
  power off the machine. Treat the bind, auth, and power code paths as
  load-bearing. See [SECURITY.md](SECURITY.md).

## Development setup

```bash
git clone https://github.com/wienerlabs/wienerenvoy
cd wienerenvoy
./scripts/dev-bootstrap.sh   # toolchain, cargo subcommands, git hooks, web deps, build
```

Run the two halves in dev:

```bash
cargo run -p wienerenvoy-daemon                 # API on http://127.0.0.1:4747
WIENERENVOY_DAEMON_URL=http://127.0.0.1:4747 pnpm -C web dev   # dashboard on :3000
```

The daemon generates a bearer token on first run. In dev, point it somewhere you
can read:

```bash
WIENERENVOY_AUTH__TOKEN_PATH=/tmp/wenvoy-token cargo run -p wienerenvoy-daemon
cargo run -p wienerenvoy-cli -- --config /dev/null token show   # or read the file
```

## Project layout

| Path | What |
|------|------|
| `crates/wienerenvoy-core` | Platform-agnostic types, control traits, state machine, config, auth |
| `crates/wienerenvoy-platform-macos` | macOS power (caffeinate/pmset) and sysinfo telemetry |
| `crates/wienerenvoy-daemon` | Axum HTTP and WebSocket server, tailnet bind, auth middleware |
| `crates/wienerenvoy-cli` | `wenvoy` control CLI |
| `web/` | Next.js dashboard (static export) |
| `install/`, `infra/` | LaunchDaemon plist and install scripts |
| `docs/adr/` | Architecture Decision Records |

## Tests

```bash
cargo nextest run --workspace        # or: cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm -C web typecheck && pnpm -C web build
```

Power actions are trait-abstracted and tested against a mock; tests never put the
real machine to sleep or shut it down. Coverage targets: M0 >= 60%, M1 >= 75% on
`wienerenvoy-core`.

## Commit format

```
<type>(<scope>): <subject>
```

Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `revert`.
Scopes: `daemon`, `core`, `platform-macos`, `cli`, `web`, `ci`, `infra`, `docs`.

Example: `feat(daemon): reject peers outside the tailnet CGNAT range`.

## Architecture Decision Records

Non-obvious choices get an ADR under `docs/adr/`. Copy `docs/adr/template.md`,
number it, and link it from your PR.

## Pull requests

Fill in the PR template (Why / What / How / Verification / Risk / Checklist).
Make sure the checklist is green before requesting review.
