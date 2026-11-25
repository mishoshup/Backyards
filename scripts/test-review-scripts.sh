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
if [[ "\$1" == "api" ]]; then exit "\${FAKE_GH_API_STATUS:-0}"; fi
if [[ "\$1" == "pr" ]]; then printf '%s\\n' "\$*" >> "${scratch_directory}/summary.log"; fi
FAKE
  chmod +x "${scratch_directory}/bin/gh"
}

violations_json() {
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
}

test_glob_conversion_is_anchored_and_segment_aware() {
  local converted
  # shellcheck source=/dev/null
  source scripts/review/review-tools.sh
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
  FAKE_GH_API_STATUS=1 run_post_findings "$(violations_json 2)"
  grep -q 'finding 1' "${scratch_directory}/summary.log" || fail_test "summary missing a finding"
}

test_overflow_findings_are_listed_in_summary() {
  : > "${scratch_directory}/summary.log"
  run_post_findings "$(violations_json 32)"
  grep -q 'finding 31' "${scratch_directory}/summary.log" || fail_test "overflow finding not listed"
  expect_equal "result line" "$(cat "${scratch_directory}/review-result-holistic")" "holistic: 32 findings (first 30 posted)"
}

main() {
  write_fake_gh
  test_allowed_rules_are_extracted
  test_unknown_rule_is_rejected
  test_glob_conversion_is_anchored_and_segment_aware
  test_zero_findings_records_all_ok
  test_missing_answer_is_recorded
  test_failed_inline_posts_fall_back_to_summary
  test_overflow_findings_are_listed_in_summary
  [ "${failures}" -eq 0 ]
}

main "$@"
