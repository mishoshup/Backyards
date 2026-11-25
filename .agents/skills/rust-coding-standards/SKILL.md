---
name: rust-coding-standards
description: Coding standards for Backyards Rust code, covering performance, async, consistency, clean design, privacy, naming, structure, ports and adapters, errors, safe destructive operations, comments, tests, and the CLI, serde, MCP, TUI and GPUI frameworks. Use when writing, modifying or reviewing any Rust in this repository.
---

# Backyards Rust Coding Standards

Each standard is one file in `rules/`. The `enforcedBy` line in each file names the lint or script that checks it. `review` means only a reviewer can check it.

## How to use

Read the rule files for every category that touches your change before writing code. Run `scripts/check.sh` before a commit.

Every rule has a `source:` line naming the lint, guideline or documentation it comes from. A rule without a source does not belong here.

## Rules

| Id | Rule | Enforced by |
|---|---|---|
| ASYNC-01 | [Only binaries build a runtime](rules/async-01-runtime-only-in-binaries.md) | scripts/check-architecture.sh (no `#[tokio::main]` in library crates) |
| ASYNC-02 | [Spawned tasks own their data](rules/async-02-send-and-static-tasks.md) | compiler; clippy::future_not_send |
| CLEAN-01 | [Compose small types instead of adding flags](rules/clean-01-composition-over-configuration.md) | review; clippy::fn_params_excessive_bools, struct_excessive_bools |
| CLEAN-02 | [A type owns its behavior](rules/clean-02-own-behavior.md) | review |
| CLEAN-03 | [Functions take what they need as arguments](rules/clean-03-context-free-contracts.md) | clippy::disallowed_methods (std::env::var, std::env::current_dir in feature crates); review |
| CLEAN-04 | [Separate deciding from doing](rules/clean-04-no-side-effect-spaghetti.md) | review |
| CLEAN-05 | [Keep state narrow and owned in one place](rules/clean-05-narrow-state.md) | review; clippy::rc_mutex, mutex_atomic |
| CLEAN-06 | [Use newtypes and enums instead of strings and flags](rules/clean-06-types-over-strings.md) | review; clippy::wildcard_enum_match_arm |
| CLEAN-07 | [Make invalid states unrepresentable](rules/clean-07-make-invalid-states-unrepresentable.md) | review; clippy::option_option, struct_field_names |
| CLI-01 | [Define the CLI with the clap derive API](rules/cli-01-derive-parser-one-enum-per-area.md) | scripts/check-architecture.sh (commands registered in one table); unit test |
| CLI-02 | [Validate input in value parsers and ValueEnum](rules/cli-02-validate-at-the-boundary.md) | review; unit tests of rejected input |
| CLI-03 | [Test the command definition](rules/cli-03-verify-the-command-definition.md) | unit test with `CommandFactory::command().debug_assert()` |
| CLI-04 | [Data on stdout, messages on stderr, stable exit codes](rules/cli-04-output-and-exit-codes.md) | review; integration tests assert stdout, stderr and the exit code |
| CLI-05 | [Every command can emit JSON](rules/cli-05-json-output-for-agents.md) | integration test per command; JSON schema generated from the types |
| COMMENTS-01 | [Comment non-obvious code only](rules/comments-01-default-to-none.md) | scripts/check-policy.sh (markers, em dashes); review for the rest |
| CONSISTENCY-01 | [Keep platform checks in one module](rules/consistency-01-isolate-platform-checks.md) | scripts/check-architecture.sh (`cfg(target_os` only under `platform/`) |
| CONSISTENCY-02 | [Name every magic value](rules/consistency-02-no-magic-values.md) | clippy::unreadable_literal; review |
| CONSISTENCY-03 | [Eliminate duplication](rules/consistency-03-no-code-duplication.md) | review; the second caller moves the helper to a shared type |
| CONSISTENCY-04 | [No unused parameters, fields or items](rules/consistency-04-no-unused-items.md) | rustc dead_code, unused_variables; clippy::unused_self, used_underscore_binding |
| CONSISTENCY-05 | [Use expect with a reason, never allow](rules/consistency-05-justify-lint-expectations.md) | clippy::allow_attributes, allow_attributes_without_reason; scripts/check-policy.sh |
| CONSISTENCY-06 | [Handle errors where they can be acted on](rules/consistency-06-proper-error-handling.md) | clippy::unwrap_used, expect_used, map_err_ignore, let_underscore_must_use |
| CONSISTENCY-09 | [Name files after the behavior they hold](rules/consistency-09-file-naming.md) | review; scripts/check-architecture.sh (no `utils.rs`, `helpers.rs`, `misc.rs`, `common.rs`) |
| CONSISTENCY-10 | [Document a public item only when the signature cannot carry the contract](rules/consistency-10-doc-comments-only-for-public-contracts.md) | clippy::missing_docs_in_private_items is off; review |
| CONSISTENCY-11 | [No TODO, FIXME, HACK or XXX](rules/consistency-11-no-marker-comments.md) | scripts/check-policy.sh; clippy::todo, unimplemented |
| CONSISTENCY-12 | [Name callbacks and handlers for the action](rules/consistency-12-handler-named-for-action.md) | review |
| CONSISTENCY-14 | [A comment states why, not what](rules/consistency-14-comment-why.md) | review |
| CONSISTENCY-15 | [Write comments in plain declarative prose](rules/consistency-15-plain-comment-style.md) | scripts/check-policy.sh (no em dashes); review |
| CONSISTENCY-16 | [No assistant or tooling jargon in code or docs](rules/consistency-16-no-ai-jargon.md) | scripts/check-policy.sh (banned phrases); review |
| CONSISTENCY-17 | [Use the correct plural in messages](rules/consistency-17-plural-form.md) | review; message tests |
| CONSISTENCY-18 | [Comment code that is not self-explanatory](rules/consistency-18-comment-non-obvious-code.md) | review |
| CONSISTENCY-19 | [Message grammar and case](rules/consistency-19-copy-grammar-and-case.md) | review; message catalog tests |
| CONSISTENCY-20 | [Name functions with a verb](rules/consistency-20-function-names.md) | review |
| ERRORS-01 | [Typed errors and no silent drops](rules/errors-01-typed-errors-no-silent-drops.md) | clippy::unwrap_used, expect_used, panic, let_underscore_must_use; review for silent drops |
| GPUI-01 | [Views call the feature crates and hold no business rules](rules/gpui-01-views-hold-no-business-logic.md) | scripts/check-architecture.sh (desktop crate depends on feature crates only); review |
| GPUI-02 | [Call cx.notify only when state changed](rules/gpui-02-notify-only-on-change.md) | review |
| GPUI-03 | [Hold or detach every Task and Subscription](rules/gpui-03-hold-or-detach-tasks-and-subscriptions.md) | must_use on `Task`; review |
| GPUI-04 | [Never block the main thread](rules/gpui-04-no-blocking-on-the-main-thread.md) | clippy::disallowed_methods; review |
| GPUI-05 | [Expose every command as an action](rules/gpui-05-actions-and-key-contexts.md) | review; test that each action has a binding |
| GPUI-06 | [Build on gpui-kit and its theme](rules/gpui-06-use-the-kit-and-its-theme.md) | scripts/check-architecture.sh (only `gpui-kit` is listed, no direct `gpui` or `gpui-shell`); review |
| MCP-01 | [Tools call the feature crates and hold no logic](rules/mcp-01-tools-are-thin-over-the-features.md) | scripts/check-architecture.sh (mcp crate depends on feature crates only); review |
| MCP-02 | [Parameters and results derive JsonSchema](rules/mcp-02-typed-schemas.md) | compiler (`schemars::JsonSchema` derive); schema snapshot test |
| MCP-04 | [Return tool failures as results and bad calls as protocol errors](rules/mcp-04-distinguish-tool-errors-from-protocol-errors.md) | unit tests of both paths |
| NAMING-01 | [Spell out identifiers](rules/naming-01-spell-out-identifiers.md) | clippy::min_ident_chars (clippy.toml allows only `id`) |
| NAMING-02 | [Prefix booleans with is, has, should or can](rules/naming-02-boolean-name-prefix.md) | review |
| NAMING-03 | [Use spec-defined terms verbatim](rules/naming-03-spec-terms-verbatim.md) | review |
| PERF-01 | [Borrow instead of cloning in loops and hot paths](rules/perf-01-borrow-do-not-clone.md) | clippy::redundant_clone, needless_pass_by_value, clone_on_ref_ptr |
| PERF-02 | [Return early before expensive work](rules/perf-02-early-return.md) | review; clippy::needless_continue |
| PERF-06 | [Never block an async task](rules/perf-06-no-blocking-in-async.md) | clippy::disallowed_methods (std::thread::sleep, std::fs in async); review |
| PERF-07 | [Never hold a lock guard across an await](rules/perf-07-no-lock-across-await.md) | clippy::await_holding_lock, await_holding_refcell_ref |
| PERF-08 | [Bound channels, input sizes and concurrency](rules/perf-08-bound-everything.md) | review; clippy::disallowed_methods (unbounded channels) |
| PERF-09 | [Release every resource through a guard](rules/perf-09-release-resources-with-guards.md) | review; real-garden tests assert nothing is left behind |
| PERF-10 | [Hoist invariant work out of loops](rules/perf-10-hoist-invariant-work.md) | review; clippy::needless_collect |
| PERF-11 | [Batch round trips and parallelize independent reads](rules/perf-11-batch-and-parallelize-reads.md) | review |
| PERF-12 | [No ad hoc sleeps or timers](rules/perf-12-no-ad-hoc-timers.md) | clippy::disallowed_methods (std::thread::sleep, tokio::time::sleep) |
| PERF-14 | [Initialize shared state once](rules/perf-14-initialize-once.md) | clippy::disallowed_types (lazy statics); review |
| PERF-15 | [Tie background tasks to a cancellation token](rules/perf-15-cancel-on-shutdown.md) | review; shutdown integration test |
| PERF-16 | [Use only cancel-safe futures inside select](rules/perf-16-select-cancel-safety.md) | review |
| PORTS-01 | [The domain layer owns the ports](rules/ports-01-features-own-the-traits.md) | review; scripts/check-architecture.sh (layer dependency rule and no cross imports) |
| PRIVACY-01 | [Mask secrets in logs, audit entries and output](rules/privacy-01-mask-sensitive-data.md) | review; redaction unit tests; clippy::dbg_macro |
| SAFETY-01 | [Destructive operations are allowlisted, dry run, audited](rules/safety-01-destructive-operations.md) | review; integration tests for dry run and allowlist |
| SAFETY-02 | [Gardens start restricted](rules/safety-02-garden-defaults.md) | unit tests on the default garden specification; integration test on a real garden |
| SERDE-01 | [Reject unknown fields in manifests and requests](rules/serde-01-deny-unknown-fields.md) | unit test that an unknown key fails; review |
| SERDE-02 | [Validate while parsing](rules/serde-02-validate-during-deserialization.md) | unit tests of rejected values |
| SERDE-03 | [Version and publish the schema](rules/serde-03-versioned-schema.md) | generated schema file is checked in and compared in CI |
| STRUCTURE-01 | [Build a layered call tree](rules/structure-01-layered-call-tree.md) | clippy::too_many_lines (30), clippy::cognitive_complexity (8); review for closures |
| STRUCTURE-02 | [One file per area, one table for commands](rules/structure-02-one-file-per-area.md) | review |
| TEST-01 | [Integration and regression tests carry the proof](rules/test-01-integration-first.md) | review; CI |
| TUI-01 | [Always restore the terminal](rules/tui-01-restore-the-terminal.md) | review; test that a panic path restores |
| TUI-02 | [Render from state and do nothing else](rules/tui-02-render-is-pure.md) | snapshot tests with `TestBackend`; review |
| TUI-03 | [Keep slow work off the draw loop](rules/tui-03-events-never-block-the-draw-loop.md) | review |
| UI-01 | [Show the right progress indicator](rules/ui-01-correct-progress-indicator.md) | review |
| UI-02 | [Long content scrolls and never overflows](rules/ui-02-scrollable-views.md) | review; TUI and desktop snapshot tests at narrow sizes |
| UI-03 | [No inline colors or sizes](rules/ui-03-no-inline-style-literals.md) | scripts/check-architecture.sh (no color or pixel literals outside `theme`) |
| UI-05 | [Reuse a shared component before writing one](rules/ui-05-reuse-components.md) | review |
| UI-06 | [Layer components and import downward only](rules/ui-06-layered-components.md) | scripts/check-architecture.sh (layer import table) |
| UI-07 | [State changes are visible without color](rules/ui-07-announce-status-and-do-not-rely-on-color.md) | review; output tests with NO_COLOR set |
| UI-08 | [Every action has a text label and a keyboard path](rules/ui-08-actions-have-labels.md) | review |
