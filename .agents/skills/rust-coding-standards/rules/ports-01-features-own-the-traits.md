---
ruleId: PORTS-01
enforcedBy: review; scripts/check-architecture.sh (layer dependency rule and no cross imports)
source: Layered architecture and the dependency inversion principle; hexagonal architecture (Cockburn); workspace AGENTS.md architecture section
---

# The domain layer owns the ports

## Requirement

A domain crate defines a trait (port) for each outside system it uses, such as a garden backend, a manifest reader, a file system, a clock or an audit log. An infrastructure crate implements the trait. Domain code must not depend on an infrastructure crate.

## Dependency rule

- Dependencies point downward only: application, infrastructure, domain, foundation.
- Crates in the same layer must not import each other.

## Specifics

- The manifest reader port is defined in `manifest`. `manifest-json` implements it. Other formats are later implementations of the same port and produce the same `Manifest`.
- BoxLite types must not appear in `gardens`. `gardens-boxlite` maps `GardenSpecification` to BoxLite options.
- Tests use in-memory test doubles of the ports. A test that starts a real garden belongs to the infrastructure crate.
- Input is validated at the boundary, and unknown fields are rejected (`#[serde(deny_unknown_fields)]`).
