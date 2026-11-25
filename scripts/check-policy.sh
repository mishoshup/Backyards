#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

failures=0

report_failure() {
  echo "policy: $1" >&2
  failures=$((failures + 1))
}

tracked_text_files() {
  git ls-files --cached --others --exclude-standard -- '*.rs' '*.md' '*.toml' '*.sh' '*.json' '*.yml'
}

check_no_marker_comments() {
  local matches
  matches=$(tracked_text_files | grep -E '\.(rs|sh|toml|yml)$' | grep -v '^scripts/check-policy.sh$' \
    | xargs grep -nE '(//|#)[[:space:]]*(TODO|FIXME|HACK|XXX)\b' 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "marker comments found:"$'\n'"$matches"
}

check_no_em_dashes() {
  local matches
  matches=$(tracked_text_files | xargs grep -n $'\xe2\x80\x94' 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "em dashes found:"$'\n'"$matches"
}

check_crates_inherit_workspace_lints() {
  local manifest
  for manifest in crates/*/Cargo.toml; do
    [ -f "$manifest" ] || continue
    grep -q '^\[lints\]' "$manifest" && grep -q 'workspace = true' "$manifest" \
      || report_failure "$manifest must contain [lints] workspace = true"
  done
}

check_no_cargo_install() {
  local matches
  matches=$(tracked_text_files | grep -v '^scripts/check-policy.sh$' | xargs grep -nE 'cargo install' 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "use cargo binstall instead of cargo install:"$'\n'"$matches"
}

check_no_unsafe_opt_out() {
  local matches
  matches=$(grep -rnE '#!?\[(allow|expect)\((clippy::(unwrap_used|expect_used|panic|todo|min_ident_chars|too_many_lines|cognitive_complexity)|unsafe_code)' crates 2>/dev/null || true)
  [ -z "$matches" ] || report_failure "lint opt-outs found:"$'\n'"$matches"
}

main() {
  check_no_marker_comments
  check_no_em_dashes
  check_crates_inherit_workspace_lints
  check_no_unsafe_opt_out
  check_no_cargo_install
  if [ "$failures" -ne 0 ]; then
    exit 1
  fi
}

main "$@"
