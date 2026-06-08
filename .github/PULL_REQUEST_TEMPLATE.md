<!-- Title format: type(scope): short imperative summary (Conventional Commits) -->

## Why

<!-- Link to issue + motivation. -->
Fixes #

## What

<!-- One-sentence summary, then bullet list of substantive changes. -->

## How

<!-- Implementation approach, notable design decisions, links to relevant ADRs. -->

## Verification

<!-- Specific steps a reviewer can take to verify behavior. Include exact
     commands so the reviewer does not have to invent them. -->

```bash
# example
cargo nextest run -p wienerenvoy-core
pnpm -C web typecheck && pnpm -C web build
```

## Risk

<!-- What could break? Security blast radius (bind/auth/power)? Be explicit. -->

## Checklist

- [ ] Tests added or updated (`cargo nextest run --workspace` green)
- [ ] `cargo fmt --all -- --check` green
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` green
- [ ] `pnpm -C web typecheck` and `pnpm -C web build` green (if web touched)
- [ ] No em dashes anywhere in the diff (code, comments, docs, UI copy)
- [ ] Documentation updated where behavior changed
- [ ] `CHANGELOG.md` entry added under `[Unreleased]`
- [ ] ADR created if architecturally significant
- [ ] No secrets in diff
- [ ] Commit messages follow Conventional Commits
