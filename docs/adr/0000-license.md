# ADR-0000: License under AGPL-3.0-or-later

- Status: Accepted
- Date: 2026-06-08
- Deciders: @kh0ra

## Context

WienerEnvoy is a network-facing, self-hosted control plane meant to be developed
in public with outside contributions. The license choice trades adoption breadth
against the guarantee that improvements made by people who run it stay open.

## Decision

License the project under **AGPL-3.0-or-later**.

## Consequences

- Anyone who runs a modified WienerEnvoy and exposes it over a network must offer
  their source. For a self-hosted server control plane, that network-use clause
  is exactly the case we care about.
- Consistent with the other Wiener Labs network-facing projects (oppolink,
  wienerlog).
- Some companies avoid AGPL software, which can narrow the corporate contributor
  pool. We accept this in exchange for keeping forks open.

## Alternatives considered

- **Apache-2.0**: maximum adoption and an explicit patent grant, but a fork could
  be taken closed-source. Rejected because the network-use guarantee matters more
  here than breadth.
- **MIT**: simplest and most permissive, but offers neither copyleft nor a patent
  grant. Rejected for the same reason.
