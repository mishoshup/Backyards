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

run_workflow_lint() {
  command -v actionlint >/dev/null 2>&1 || { echo "actionlint is required; install it with brew install actionlint" >&2; exit 1; }
  actionlint .github/workflows/*.yml
}

run_workflow_policy_check() {
  scripts/check-workflows.sh
}

run_shell_check() {
  scripts/check-shell.sh
}

run_review_script_tests() {
  scripts/test-review-scripts.sh
}

main() {
  run_workflow_lint
  run_workflow_policy_check
  run_shell_check
  run_review_script_tests
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
