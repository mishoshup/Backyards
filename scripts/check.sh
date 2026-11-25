#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

run_formatting_check() {
  cargo fmt --all -- --check
}

run_lint_check() {
  cargo clippy --workspace --all-targets -- -D warnings
}

run_tests() {
  cargo test --workspace
}

run_dependency_check() {
  if command -v cargo-deny >/dev/null 2>&1; then
    cargo deny check
  else
    echo "cargo-deny is not installed; dependency check skipped" >&2
  fi
}

run_policy_check() {
  scripts/check-policy.sh
  scripts/check-architecture.sh
}

main() {
  run_policy_check
  if [ -d crates ] && [ -n "$(ls -A crates 2>/dev/null)" ]; then
    run_formatting_check
    run_lint_check
    run_tests
    run_dependency_check
  else
    echo "no crates yet; Rust checks skipped" >&2
  fi
}

main "$@"
