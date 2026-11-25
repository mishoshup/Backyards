# Backyards Engineering Specification

This document defines the requirements that every change to this repository must satisfy. The key words "must", "must not" and "may" are used in their normative sense. Automated checks enforce a requirement wherever one exists. Prose guidance applies only after those checks.

## 1. Philosophy

A computer used for several kinds of work degrades without anyone deciding that it should. Tools accumulate, versions conflict, caches grow, and work for one purpose gains access to data that belongs to another. Backyards treats this degradation as a design problem. The machine is infrastructure, and infrastructure is declared, not accumulated.

The manifest is the only authority on what the machine should be. The state of the machine is observed and compared with the manifest, and any difference is resolved by a plan that a person or an agent can read before it runs. A change that is not in the manifest is drift, and drift is reported and not silently accepted. Because the declaration is the authority, the machine can be rebuilt, audited and explained.

Isolation is the default, and access is granted deliberately. Work and personal life are different domains with different trust levels and different consequences. Employer code, client credentials and professional accounts must not be reachable from personal browsing, entertainment or experiments, and personal files must not be exposed to work tooling. Each domain runs in its own environment with the smallest set of permissions that lets it function, and an environment can reach another environment's data only when the manifest says so. A mistake or a compromise is then limited to the domain where it happened.

Resources are finite and shared. A workload that is not bounded will consume what other workloads need, so limits on processor, memory and disk are part of the declaration and are enforced by the system. Storage is treated the same way. Data that is no longer used is removed automatically on a defined retention schedule, and the removal is visible, bounded and reversible where possible. Cleanup is routine and performed by the system, not an occasional manual effort.

Secrets belong to an identity and stay with it. A credential is stored where its owner can use it, is referenced by name in the manifest, and is never copied into environments that do not need it, written to a log or shown in output.

Software agents are primary users of this system. They act quickly and at scale, and they cannot be relied on to notice an unusual situation. The system therefore makes every operation predictable: a dry run is the default, destructive operations apply only to resources that Backyards created, every step is reported, and every output is available in a machine-readable form. An agent that follows the documented interface cannot cause harm that the manifest did not permit.

Nothing is dropped silently. An item that is skipped, an error that is ignored or a feature that is disabled is recorded and reported, because an operator cannot trust a system whose failures are invisible.

When goals conflict, security takes precedence over speed, and speed takes precedence over resource use. A secure system that is too slow to use will be bypassed, and a fast system that consumes the machine it runs on has failed its purpose.

## 2. Scope

1. BoxLite is the only garden backend in the first version. Other backends are out of scope and remain behind the backend port.
2. The command, the package and the application crates share the name `backyards`.
3. Interfaces are delivered in this order: CLI, MCP server, terminal UI, desktop application. Each is a thin front end over the domain crates and contains no business logic. The frameworks are clap (CLI), rmcp (MCP), ratatui (terminal UI) and GPUI through the `gpui-kit` crate (desktop, without `gpui-shell`).
4. Every Rust tool must be installed with `cargo binstall` (prebuilt binaries). Tools must not be compiled from source, locally or in CI. `scripts/check-policy.sh` enforces this requirement.
5. All dependencies and tools must be open source. The system must not require a paid tool or a network service that the user does not operate.

## 3. Architecture

### 3.1 Workspace

Rust stable, edition 2024. Workspace crates reside in `crates/`. Each crate has one responsibility.

### 3.2 Layers and dependency rule

The code is arranged in four layers. A crate may depend only on a layer below its own.

| Layer | Crates | Responsibility |
| --- | --- | --- |
| Application | `backyards`, `backyards-mcp`, `backyards-tui`, `backyards-desktop` | User interfaces and wiring. Contains no business logic. |
| Infrastructure | `gardens-boxlite`, `manifest-json` | Implements ports against outside systems. |
| Domain | `gardens`, `account`, `agents`, `clean`, `secrets`, `disk`, `manifest` | Business rules and the ports (traits) that infrastructure implements. |
| Foundation | `kernel` | Small shared primitives with no domain vocabulary. |

1. Cross imports are prohibited. Crates in the same layer must not depend on each other. A domain crate must not import another domain crate, and an infrastructure crate must not import another infrastructure crate. The application layer connects two domain crates by passing the output of one to the other.
2. The domain layer defines the ports. The infrastructure layer implements them (dependency inversion). Domain code must not name an infrastructure crate. An infrastructure crate may depend on the single domain crate whose port it implements.
3. Each domain crate is a self-contained module. It must build, test and run without any other domain crate, and it owns its own manifest section.
4. Internal crates omit the product prefix and are not published. Only the application crates carry the prefix.
5. `scripts/check-architecture.sh` fails the gate when a crate imports a crate that its layer does not permit.

### 3.3 Reconciliation model

1. Planning is a pure function from the desired state and the observed state to a list of actions (functional core, imperative shell). Only the apply step performs side effects.
2. The sequence is: read the manifest (desired state), read the backend (observed state), compute a plan, apply the plan.
3. Apply must be idempotent. A second run against an unchanged system must change nothing.

### 3.4 Domain model

1. Domain types are independent of any backend. `GardenName` and `GardenSpecification` carry Backyards rules only.
2. A backend adapter maps domain types to the identifiers of its backend, tags every garden it creates as managed, and lists only managed gardens.
3. The backyard (the second macOS user) is optional. When the manifest declares no backyard, every other feature acts on the current user, and its tests run without a backyard.

### 3.5 Manifest

1. The manifest is a versioned envelope: `schema_version` and named sections.
2. Each domain crate owns and validates its own section.
3. A section that no domain crate claims must be reported as an error and must not be dropped.
4. The first manifest format is JSON, read through the `ManifestReader` port. Other formats are later implementations of the same port.

## 4. Engineering requirements

1. Identifiers must not be abbreviated. A term defined by a specification is used verbatim, and its expansion is given in a comment.
2. Illegal states must be unrepresentable. Input is parsed into validated types at the boundary (`GardenName`, `NetworkPolicy`), and those types are passed inward. Raw strings must not travel through domain code.
3. Every change is structured as a layered call tree. A top-level function delegates to a small number of named helper functions, which delegate further. Long closures and flat inline chains are prohibited. Business rules must not reside in a closure.
4. Each area or behavior has its own file, named after it. Commands are registered in one table, grouped by area.
5. Library crates use typed errors (`thiserror`). The constructs `unwrap`, `expect`, `panic!`, `todo!` and `unsafe` are prohibited. `anyhow` is permitted only in the `backyards` binary crate.
6. A silent drop is a defect. Every skipped item, ignored error and disabled path must be recorded and reported.
7. Library behavior must be verified against current crate source or upstream documentation before it is relied upon, and the source must be stated. The BoxLite documentation site lags the crate, so the source of the pinned version is authoritative.
8. Operations must use the least compute and memory that preserve correctness. The trade-off must be stated when a cache, an index or a background task is added.

## 5. Terminology

Backyards names its own domain nouns with garden words under the following rules.

1. Only a noun that names a Backyards concept with no standard term receives a garden word.
2. The following are never renamed: verbs and commands (`plan`, `apply`, `list`, `remove`), states and adjectives (`disabled`, `enabled`), standard technical terms (`manifest`, `network`, `image`, `schema_version`) and external product names (`BoxLite`).
3. One concept has one word, and one word has one concept. Each garden word must be defined in the table below before use.
4. A garden word is added only when its concept exists in the code.
5. The backend adapter translates between a garden word and the term of its backend. Domain crates and front ends must not use the backend term.

| Term | Definition |
| --- | --- |
| backyard | One isolated macOS user that Backyards creates and manages. It contains gardens. Operating system term: user account. |
| garden | One isolated environment that Backyards creates, tracks and removes. Backend term: box. |

## 6. Safety requirements

1. The principles of least privilege and fail-closed apply. A setting looser than the default requires an explicit manifest entry. Unknown or invalid input must be rejected.
2. A destructive step must act only on managed resources (allowlist), run as a dry run by default, write an append-only audit log entry, and report after each step. These requirements bound the blast radius of an incorrect plan.
3. Retention for caches and images is 7 days unless the manifest specifies otherwise.
4. The garden defaults are: no outbound network (`NetworkSpec::Disabled` in the BoxLite adapter), `SecurityOptions::enabled()` (full isolation, the crate default), ports bound to `127.0.0.1`, and a state directory with mode 700.
5. Tests must not access the real home directory. They use a temporary directory and remove it afterwards.
6. Secrets must not be printed or logged.

## 7. Comments

1. A comment is permitted only on code that is not self-evident: a constraint, a workaround for a library or platform defect, an ordering dependency, or a deliberate trade-off. The comment states the reason. Self-evident code carries no comment.
2. A comment must not restate the code, record history or describe future work.
3. The markers `TODO`, `FIXME`, `HACK` and `XXX` are prohibited. Missing behavior is implemented with a test, or tracked as an issue.
4. A rule is expressed first in a name, a type, a lint or a test. A comment is the last resort.
5. When review finds the same defect twice, a deterministic check must be added or strengthened. An additional prose rule is not an acceptable first response.

## 8. Writing

Documentation, comments, commit messages and reports use plain, concise, declarative prose. They contain no slogans, no narrative framing, no em dashes and no comparisons with rejected alternatives. Planned work is labeled as planned. The user's own wording is not quoted in documentation.

## 9. Quality gates

1. `scripts/check.sh` must pass before every commit. It runs formatting, Clippy with warnings denied, tests, dependency checks (cargo-deny), and the policy and architecture checks. The pre-commit hook and CI run the same command, so a local result and a remote result are equivalent.
2. After each edit, `cargo clippy --workspace --all-targets` must be run for the affected crates.
3. Tests that start a real garden run only on macOS with Apple silicon and remove every garden they create.
4. Test weight, in descending order: integration tests against real behavior, then unit tests.
5. Planned gates, to be added when the crates exist: diff coverage of changed lines (cargo-llvm-cov), a generated JSON Schema compared with the checked-in contract, and a release-build startup check.

## 10. Version control

Every change reaches `main` through a pull request. Direct pushes to `main` are prohibited.

1. Each concern has one branch and one pull request, branched from an up-to-date `main`. The branch name is `<type>/<short-description>` in lowercase with hyphens. `<type>` is one of `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`, `ci`, `build`.
2. The pull request title and the commit subject use `<type>(<scope>): <subject>`: lowercase, imperative mood, at most 72 characters, no trailing period. The title type matches the branch type. Pull requests are merged by squash, so the title becomes the commit on `main`.
3. The pull request description completes the template (problem, change, verification). Every checklist item is checked or marked n/a.
4. Commit messages are plain, carry no `Co-Authored-By` or `Claude-Session` trailers, and use the user's own git identity.
5. A commit, a push or a pull request requires explicit approval for that batch of changes. Only named files are staged, and unrelated changes are checked first.
6. GitHub operations run as the `mishoshup` account. `gh auth status` must be checked first.
7. A branch is pushed with an explicit refspec (`git push origin HEAD:refs/heads/<branch>`). Force pushes and pushes to `main` are prohibited.
8. A pull request is updated by adding commits or by rebasing onto `main`. `main` must not be merged into the branch.
9. ADRs, research notes and design documents remain local in `~/Developer/backyards-local/` and must not be added to the repository.

## 11. Skills and reviewers

Detailed procedures are in `.agents/skills/`. The matching skill must be read before the corresponding work.

| Skill | Applies to |
| --- | --- |
| `rust-coding-standards` | Writing, changing or reviewing Rust. One file per rule in `rules/`, each with a source and an enforcing check. |
| `boxlite-adapter` | Changes to the BoxLite backend or to any garden option. |
| `real-box-testing` | Writing or running tests that start a real garden. |
| `ship-change` | Taking completed work to a reviewed pull request. |

Reviewer agents are in `.claude/agents/`. The command `/review-changes` runs them on the current diff. `.claude/skills` is a symbolic link to `.agents/skills`.

## 12. Editing requirements

1. Every existing comment that remains valid is retained. A comment invalidated by a change is updated.
2. Formatting, hooks and lints must not be skipped to pass a change. The cause is corrected.
3. A bug fix includes a test that reproduces the bug. A feature includes tests.
4. Generated code must be read before submission. The author is responsible for every line.

## 13. Agent guidance

1. Agent guidance supplies judgment after the deterministic checks and does not replace them.
2. A report or review finding must be confirmed against the source before action is taken.
