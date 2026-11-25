#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
# shellcheck source=/dev/null
source scripts/review/review-tools.sh

readonly base_url="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}"
readonly request_timeout_seconds=600
readonly maximum_rounds=14
readonly model="${REVIEW_MODEL_HOLISTIC:-deepseek-v4-pro}"
readonly tool_definitions="scripts/review/tool-definitions.json"

build_request_body() {
  jq -n --arg model "${model}" --arg choice "$2" --slurpfile messages "$1" --slurpfile tools "${tool_definitions}" \
    '{model: $model, messages: $messages[0], tools: $tools[0], tool_choice: $choice}'
}

call_model() {
  build_request_body "$1" "$2" | curl -fsS --max-time "${request_timeout_seconds}" "${base_url}/chat/completions" \
    -H "Authorization: Bearer ${DEEPSEEK_API_KEY}" -H "Content-Type: application/json" --data-binary @-
}

append_message() {
  local messages_file="$1" message_json="$2" updated
  updated=$(jq --argjson message "${message_json}" '. + [$message]' "${messages_file}")
  printf '%s\n' "${updated}" > "${messages_file}"
}

run_tool_call() {
  local call_json="$1" name arguments output
  name=$(jq -r '.function.name' <<<"${call_json}")
  arguments=$(jq -r '.function.arguments' <<<"${call_json}")
  echo "review: holistic tool ${name} ${arguments:0:120}" >&2
  output=$(run_tool "${name}" "${arguments}" 2>&1 || true)
  jq -n --arg id "$(jq -r '.id' <<<"${call_json}")" --arg content "${output}" '{role: "tool", tool_call_id: $id, content: $content}'
}

answer_tool_calls() {
  local messages_file="$1" assistant_message="$2" call_json
  while IFS= read -r call_json; do
    append_message "${messages_file}" "$(run_tool_call "${call_json}")"
  done < <(jq -c '.tool_calls[]' <<<"${assistant_message}")
}

extract_violations() {
  local content="$1"
  grep -v '^```' <<<"${content}" | jq -ce '.violations | arrays | {violations: .}' 2>/dev/null
}

choose_tool_policy() {
  if (( $1 >= maximum_rounds )); then echo none; else echo auto; fi
}

run_rounds() {
  local messages_file="$1" round response assistant_message result
  for round in $(seq 1 "${maximum_rounds}"); do
    echo "review: holistic round ${round}, ${SECONDS}s elapsed" >&2
    response=$(call_model "${messages_file}" "$(choose_tool_policy "${round}")")
    assistant_message=$(jq -c '.choices[0].message' <<<"${response}")
    append_message "${messages_file}" "${assistant_message}"
    if [[ "$(jq '(.tool_calls // []) | length' <<<"${assistant_message}")" -gt 0 ]]; then
      answer_tool_calls "${messages_file}" "${assistant_message}"
    elif result=$(extract_violations "$(jq -r '.content // ""' <<<"${assistant_message}")"); then
      printf '%s\n' "${result}"
      return 0
    else
      append_message "${messages_file}" '{"role":"user","content":"Respond with the JSON object only."}'
    fi
  done
  echo "review: holistic produced no valid answer in ${maximum_rounds} rounds" >&2
  return 1
}

main() {
  local messages_file
  messages_file=$(mktemp)
  jq -n --arg system "$(scripts/review/build-review-prompt.sh holistic)" --arg diff "$(scripts/review/read-pull-request-diff.sh)" \
    '[{role: "system", content: $system}, {role: "user", content: ("Review this diff. Use the tools to read the repository and documentation before you report a finding. Answer in JSON.\n\n" + $diff)}]' > "${messages_file}"
  run_rounds "${messages_file}"
}

main "$@"
