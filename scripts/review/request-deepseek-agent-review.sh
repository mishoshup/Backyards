#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
# shellcheck source=/dev/null
source scripts/review/review-tools.sh

readonly base_url="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}"
readonly request_timeout_seconds=120
readonly maximum_rounds=8
readonly model="${REVIEW_MODEL_HOLISTIC:-deepseek-flash}"
readonly tool_definitions="scripts/review/tool-definitions.json"

build_request_body() {
  jq -n --arg model "${model}" --arg choice "$2" --slurpfile messages "$1" --slurpfile tools "${tool_definitions}" \
    '{model: $model, messages: $messages[0], tools: $tools[0], tool_choice: $choice}'
}

post_chat_completion() {
  curl -sS --max-time "${request_timeout_seconds}" -w '\n%{http_code}' "${base_url}/chat/completions" \
    -H "Authorization: Bearer ${DEEPSEEK_API_KEY}" -H "Content-Type: application/json" --data-binary @-
}

call_model() {
  local reply status
  reply=$(build_request_body "$1" "$2" | post_chat_completion)
  status=${reply##*$'\n'}
  if [ "${status}" != 200 ]; then
    echo "review: DeepSeek returned HTTP ${status}: ${reply%$'\n'*}" >&2
    return 1
  fi
  printf '%s\n' "${reply%$'\n'*}"
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

isolate_json_object() {
  awk 'BEGIN { depth = 0 } { for (i = 1; i <= length($0); i++) { c = substr($0, i, 1); if (c == "{") { depth++; started = 1 } if (started) printf "%s", c; if (c == "}") { depth--; if (started && depth == 0) { exit } } } if (started) printf "\n" }'
}

parse_violations() {
  jq -ce '.violations | arrays | {violations: .}' 2>/dev/null
}

extract_violations() {
  parse_violations <<<"$1" || isolate_json_object <<<"$1" | parse_violations
}

announce_remaining_rounds() {
  if (( $2 >= maximum_rounds - 1 )); then
    append_message "$1" '{"role":"user","content":"Stop reading. Answer now with the JSON object only, listing every finding you have confirmed."}'
  fi
}

choose_tool_policy() {
  if (( $1 >= maximum_rounds )); then echo none; else echo auto; fi
}

run_rounds() {
  local messages_file="$1" round response assistant_message result
  for round in $(seq 1 "${maximum_rounds}"); do
    echo "review: holistic round ${round}, ${SECONDS}s elapsed" >&2
    announce_remaining_rounds "${messages_file}" "${round}"
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

messages_file=""

remove_messages_file() {
  rm -f -- "${messages_file}"
}

main() {
  messages_file=$(mktemp)
  trap remove_messages_file EXIT
  jq -n --arg system "$(scripts/review/build-review-prompt.sh holistic)" --arg diff "$(scripts/review/read-pull-request-diff.sh)" \
    '[{role: "system", content: $system}, {role: "user", content: ("Review this diff. Use the tools to read the repository and documentation before you report a finding. Answer in JSON.\n\n" + $diff)}]' > "${messages_file}"
  run_rounds "${messages_file}"
}

main "$@"
