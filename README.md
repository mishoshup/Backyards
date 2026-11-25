# Backyards

## Purpose

Backyards reconciles one Mac toward a declared desired state. A manifest describes the machine. Backyards computes a plan against the observed state, applies it, and removes stale resources. The target users are software agents and the developers who supervise them.

## Status

Early development. No release exists.

## Interfaces

| Interface | Status |
| --- | --- |
| CLI (`backyards`) | Planned first |
| MCP server | Planned |
| Terminal UI | Planned |
| Desktop application | Planned |

## Design constraints

- The manifest is the single source of truth.
- Destructive operations run as a dry run by default, act only on managed resources, and write an audit entry.
- Priorities, in order: security, speed, low resource use.

## Contributing

Every change reaches `main` through a pull request. The rules for contributors and agents are in `AGENTS.md`.

## License

AGPL-3.0-only. The full text is in `LICENSE`.
