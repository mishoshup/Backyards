#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

strip_front_matter() {
  awk 'BEGIN{fence=0} /^---$/ && fence<2 {fence++; next} fence>=2 || fence==0 {print}' "$1"
}

print_reviewer_instructions() {
  case "$1" in
    inline) strip_front_matter .claude/agents/code-inline-reviewer.md ;;
    holistic) strip_front_matter .claude/agents/holistic-reviewer.md ;;
    *) echo "review: unknown mode $1" >&2; exit 2 ;;
  esac
}

print_rule_files() {
  local rule_file
  for rule_file in .agents/skills/rust-coding-standards/rules/*.md; do
    printf '\n### %s\n' "$rule_file"
    cat "$rule_file"
  done
}

print_output_contract() {
  cat <<'CONTRACT'

Respond with JSON only, in this shape:
{"violations":[{"ruleId":"<rule identifier, or the area name for a holistic finding>","path":"<file path>","line":<line number in the new file>,"body":"<rule identifier, defect and smallest correction>"}]}
The body of an inline violation must contain its rule identifier.
Do not report a model name, API name, flag or library behavior as wrong unless the diff itself shows the failure.
Always answer with the JSON object. Use {"violations":[]} only as a definitive statement that you checked the diff and found nothing to report. Report only lines present in the diff.
CONTRACT
}

main() {
  print_reviewer_instructions "$1"
  if [ "$1" = "inline" ]; then
    print_rule_files
  fi
  print_output_contract
}

main "$@"
