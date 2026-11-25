#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

readonly verb_file=scripts/shell-function-verbs.txt
failures=0

report_failure() {
  echo "shell: $1" >&2
  failures=$((failures + 1))
}

list_shell_files() {
  git ls-files --cached --others --exclude-standard -- '*.sh'
}

run_shellcheck() {
  command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck is required; install it with brew install shellcheck" >&2; exit 1; }
  list_shell_files | xargs shellcheck -x || report_failure "shellcheck reported problems"
}

list_function_names() {
  list_shell_files | xargs grep -HoE '^[a-z_]+\(\) *\{' | sed -E 's/\(\) *\{//'
}

check_function_names_start_with_a_verb() {
  local entry file name verb
  while IFS= read -r entry; do
    file=${entry%%:*}
    name=${entry#*:}
    verb=${name%%_*}
    grep -qxF "${verb}" "${verb_file}" || report_failure "${file}: function ${name} must start with a verb from ${verb_file}"
  done < <(list_function_names)
}

main() {
  run_shellcheck
  check_function_names_start_with_a_verb
  [ "${failures}" -eq 0 ]
}

main "$@"
