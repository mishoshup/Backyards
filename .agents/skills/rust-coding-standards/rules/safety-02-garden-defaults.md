---
ruleId: SAFETY-02
enforcedBy: unit tests on the default garden specification; integration test on a real garden
source: BoxLite crate source 0.14.1 (SecurityOptions, NetworkSpec, PortSpec); workspace AGENTS.md safety rules
---

# Gardens start restricted

## Requirement

The default garden specification must set all of the following:

- No outbound network (`NetworkSpec::Disabled`). An allow list requires an explicit manifest entry.
- `SecurityOptions::enabled()`, which is full isolation and the crate default.
- Published ports bound to `127.0.0.1` only.
- CPU, memory and disk limits taken from the manifest.
- A BoxLite state directory with mode 700.

## Constraint

Do not rely on `inbound: disabled`. A default garden reaches the host loopback at `192.168.127.254`. An allow list must not include that address unless the manifest states it.

## Deviation

A looser setting requires a named manifest entry and must appear in the plan output.
