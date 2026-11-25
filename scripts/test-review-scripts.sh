#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

readonly workspace_root="${PWD}"
scratch_directory=$(mktemp -d)
readonly scratch_directory
failures=0

remove_scratch_directory() {
  rm -rf -- "${scratch_directory}"
}
trap remove_scratch_directory EXIT

# shellcheck source=/dev/null
source scripts/review/review-tools.sh

fail_test() {
  echo "review script test failed: $1" >&2
  failures=$((failures + 1))
}

expect_equal() {
  [[ "$2" == "$3" ]] || fail_test "$1: expected '$3' but got '$2'"
}

write_fake_gh() {
  mkdir -p "${scratch_directory}/bin"
  cat > "${scratch_directory}/bin/gh" <<FAKE
#!/usr/bin/env bash
if [[ "\$1" == "api" ]]; then cat > /dev/null; exit "\${FAKE_GH_API_STATUS:-0}"; fi
if [[ "\$1" == "pr" ]]; then printf '%s\\n' "\$*" >> "${scratch_directory}/summary.log"; fi
FAKE
  chmod +x "${scratch_directory}/bin/gh"
}

build_violations_json() {
  jq -n --argjson count "$1" '{violations: [range($count) | {path: "a.rs", line: 1, body: ("finding " + (. | tostring)), ruleId: "ERRORS-01"}]}'
}

run_post_findings() {
  RUNNER_TEMP="${scratch_directory}" PATH="${scratch_directory}/bin:${PATH}" PULL_REQUEST_NUMBER=1 REPOSITORY=o/r \
    GITHUB_REPOSITORY=o/r REVIEW_HEAD_SHA=abcdef1234 ALLOWED_RULES_FILE="${scratch_directory}/rules.txt" \
    "${workspace_root}/scripts/review/post-review-findings.sh" holistic <<<"$1"
}

test_allowed_rules_are_extracted() {
  GITHUB_ENV="${scratch_directory}/env" .github/scripts/review/extract-allowed-rules.sh \
    .agents/skills/rust-coding-standards/rules "${scratch_directory}/rules.txt"
  grep -qxF 'ERRORS-01' "${scratch_directory}/rules.txt" || fail_test "ERRORS-01 missing from extracted rules"
}

test_unknown_rule_is_rejected() {
  if ALLOWED_RULES_FILE="${scratch_directory}/rules.txt" REVIEW_HEAD_SHA=abcdef1234 GITHUB_REPOSITORY=o/r \
    .github/scripts/review/create-inline-comment.sh 1 a.rs "body" 1 inline NOPE-99 2>/dev/null; then
    fail_test "unknown rule was accepted"
  fi
  local message
  message=$(ALLOWED_RULES_FILE="${scratch_directory}/rules.txt" REVIEW_HEAD_SHA=abcdef1234 GITHUB_REPOSITORY=o/r \
    .github/scripts/review/create-inline-comment.sh 1 a.rs "body" 1 inline NOPE-99 2>&1 || true)
  grep -q "not in the allowed list" <<<"${message}" || fail_test "rejection did not name the allowlist"
}

test_known_rule_is_accepted() {
  ALLOWED_RULES_FILE="${scratch_directory}/rules.txt" REVIEW_HEAD_SHA=abcdef1234 GITHUB_REPOSITORY=o/r PATH="${scratch_directory}/bin:${PATH}" \
    .github/scripts/review/create-inline-comment.sh 1 a.rs "body" 1 inline ERRORS-01 || fail_test "known rule was rejected"
}

test_glob_conversion_is_anchored_and_segment_aware() {
  local converted
  converted=$(convert_glob_to_pattern 'crates/*/Cargo.toml')
  grep -qE "${converted}" <<<"crates/a/Cargo.toml" || fail_test "glob did not match a direct child"
  if grep -qE "${converted}" <<<"crates/a/b/Cargo.toml"; then fail_test "single star crossed a directory"; fi
}

test_zero_findings_records_all_ok() {
  run_post_findings '{"violations":[]}'
  expect_equal "zero findings result" "$(cat "${scratch_directory}/review-result-holistic")" "holistic: no findings, all ok"
}

test_missing_answer_is_recorded() {
  if run_post_findings 'not json' 2>/dev/null; then fail_test "invalid answer was accepted"; fi
  expect_equal "missing answer result" "$(cat "${scratch_directory}/review-result-holistic")" "holistic: no answer"
}

test_failed_inline_posts_fall_back_to_summary() {
  : > "${scratch_directory}/summary.log"
  FAKE_GH_API_STATUS=1 run_post_findings "$(build_violations_json 2)"
  grep -q 'finding 1' "${scratch_directory}/summary.log" || fail_test "summary missing a finding"
}

test_overflow_findings_are_listed_in_summary() {
  : > "${scratch_directory}/summary.log"
  run_post_findings "$(build_violations_json 32)"
  grep -q 'finding 31' "${scratch_directory}/summary.log" || fail_test "overflow finding not listed"
  expect_equal "result line" "$(cat "${scratch_directory}/review-result-holistic")" "holistic: 32 findings (first 30 posted)"
}

test_reviewable_paths_include_policy_files() {
  local path
  for path in crates/a/src/lib.rs clippy.toml deny.toml rust-toolchain.toml AGENTS.md scripts/check.sh .github/workflows/ci.yml; do
    printf '%s\n' "${path}" > "${scratch_directory}/changed.txt"
    .github/scripts/review/select-reviewable-paths.sh "${scratch_directory}/changed.txt" || fail_test "${path} was not selected for review"
  done
}

test_unrelated_paths_are_not_reviewable() {
  printf '%s\n' README.md docs/notes.txt > "${scratch_directory}/changed.txt"
  if .github/scripts/review/select-reviewable-paths.sh "${scratch_directory}/changed.txt"; then fail_test "unrelated paths were selected"; fi
}

test_status_state_is_validated() {
  local message
  message=$(GITHUB_REPOSITORY=o/r PATH="${scratch_directory}/bin:${PATH}" \
    .github/scripts/review/record-review-complete.sh "$(printf 'a%.0s' {1..40})" ai-review-completed/deepseek/pr-1 desc bogus 2>&1 || true)
  grep -q "state must be" <<<"${message}" || fail_test "invalid status state was accepted"
}

test_tool_output_is_truncated_with_a_marker() {
  local output
  head -c 300000 /dev/zero | tr '\0' 'x' > "${scratch_directory}/large.txt"
  output=$(limit_output < "${scratch_directory}/large.txt")
  grep -q "truncated" <<<"${output}" || fail_test "oversized tool output had no truncation marker"
}

test_search_with_many_matches_is_not_a_tool_error() {
  local output
  mkdir -p "${scratch_directory}/many"
  seq 1 500 | sed 's/^/match /' > "${scratch_directory}/many/file.txt"
  output=$(cd "${scratch_directory}/many" && set -o pipefail && search_repository '{"pattern":"match"}'; echo "status:$?")
  grep -q 'status:0' <<<"${output}" || fail_test "truncated search returned a non-zero status"
}


main() {
  write_fake_gh
  test_allowed_rules_are_extracted
  test_unknown_rule_is_rejected
  test_known_rule_is_accepted
  test_glob_conversion_is_anchored_and_segment_aware
  test_zero_findings_records_all_ok
  test_missing_answer_is_recorded
  test_failed_inline_posts_fall_back_to_summary
  test_overflow_findings_are_listed_in_summary
  test_reviewable_paths_include_policy_files
  test_unrelated_paths_are_not_reviewable
  test_status_state_is_validated
  test_tool_output_is_truncated_with_a_marker
  test_search_with_many_matches_is_not_a_tool_error
  [ "${failures}" -eq 0 ]
}

main "$@"
