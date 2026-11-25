#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

failures=0

report_failure() {
  echo "architecture: $1" >&2
  failures=$((failures + 1))
}

rust_files() {
  find crates -name '*.rs' -not -path '*/target/*' 2>/dev/null || true
}

check_no_catch_all_file_names() {
  local matches
  matches=$(rust_files | grep -E '/(utils|helpers|misc|common)\.rs$' || true)
  [ -z "$matches" ] || report_failure "catch-all file names:"$'\n'"$matches"
}

check_runtime_only_in_binaries() {
  local matches
  matches=$(rust_files | grep -v '/main\.rs$' | xargs grep -nE '#\[tokio::main\]' 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "runtime macro outside a binary entry point:"$'\n'"$matches"
}

check_platform_checks_are_isolated() {
  local matches
  matches=$(rust_files | grep -v '/platform/' | xargs grep -nE 'cfg\(.*target_os' 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "target_os checks outside a platform module:"$'\n'"$matches"
}

check_file_length_limits() {
  local file lines
  while IFS= read -r file; do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 400 ]; then
      report_failure "$file has $lines lines, hard limit is 400"
    elif [ "$lines" -gt 300 ]; then
      echo "architecture: warning: $file has $lines lines, soft limit is 300" >&2
    fi
  done < <(rust_files)
}

check_anyhow_only_in_binary() {
  local manifest matches
  for manifest in crates/*/Cargo.toml; do
    [ -f "$manifest" ] || continue
    [ "$(basename "$(dirname "$manifest")")" = "backyards" ] && continue
    matches=$(grep -nE '^\s*anyhow\s*=' "$manifest" || true)
    [ -z "$matches" ] || report_failure "$manifest uses anyhow outside the binary crate:"$'\n'"$matches"
  done
}

check_no_gpui_shell() {
  local matches
  matches=$(grep -rnE '^\s*gpui-shell\s*=' Cargo.toml crates/*/Cargo.toml 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "gpui-shell is not allowed:"$'\n'"$matches"
}

crate_names() {
  local directory
  for directory in crates/*/; do
    basename "$directory"
  done
}

allowed_internal_dependencies() {
  local crate="$1" candidate
  case "$crate" in
    kernel) return 0 ;;
    manifest) echo "kernel"; return 0 ;;
    manifest-json) echo "kernel manifest"; return 0 ;;
    backyards | backyards-*) crate_names; return 0 ;;
  esac
  echo "kernel"
  for candidate in $(crate_names); do
    case "$crate" in "$candidate"-*) echo "$candidate" ;; esac
  done
}

declared_internal_dependencies() {
  local manifest="$1" name
  for name in $(crate_names); do
    if grep -qE "^${name}(\.workspace)?[[:space:]]*=" "$manifest"; then
      echo "$name"
    fi
  done
}

check_features_do_not_depend_on_each_other() {
  local crate dependency allowed
  for crate in $(crate_names); do
    allowed=" $(allowed_internal_dependencies "$crate" | tr '\n' ' ') "
    for dependency in $(declared_internal_dependencies "crates/$crate/Cargo.toml"); do
      case "$allowed" in
        *" $dependency "*) ;;
        *) report_failure "crates/$crate depends on $dependency, features may depend only on kernel and their own adapters" ;;
      esac
    done
  done
}

check_front_ends_depend_on_core_only() {
  local manifest matches
  for manifest in crates/backyards-mcp/Cargo.toml crates/backyards-desktop/Cargo.toml crates/backyards-tui/Cargo.toml; do
    [ -f "$manifest" ] || continue
    matches=$(grep -nE '^\s*(gardens-boxlite|manifest-json)' "$manifest" || true)
    [ -z "$matches" ] || report_failure "$manifest depends on an adapter:"$'\n'"$matches"
  done
}

main() {
  check_no_catch_all_file_names
  check_runtime_only_in_binaries
  check_platform_checks_are_isolated
  check_file_length_limits
  check_anyhow_only_in_binary
  check_no_gpui_shell
  check_front_ends_depend_on_core_only
  check_features_do_not_depend_on_each_other
  if [ "$failures" -ne 0 ]; then
    exit 1
  fi
}

main "$@"
