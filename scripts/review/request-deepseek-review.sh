#!/usr/bin/env bash
set -euo pipefail

base_url="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}"

select_model() {
  case "$1" in
    inline) echo "${REVIEW_MODEL_INLINE:-deepseek-flash}" ;;
    holistic) echo "${REVIEW_MODEL_HOLISTIC:-deepseek-v4-pro}" ;;
  esac
}

build_request_body() {
  local model="$1" system_prompt="$2" diff_text="$3"
  jq -n --arg model "$model" --arg system "$system_prompt" --arg diff "$diff_text" '{
    model: $model,
    response_format: {type: "json_object"},
    messages: [
      {role: "system", content: $system},
      {role: "user", content: ("Review this diff and answer in JSON.\n\n" + $diff)}
    ]
  }'
}

send_request() {
  curl -fsS --max-time 600 "${base_url}/chat/completions" \
    -H "Authorization: Bearer ${DEEPSEEK_API_KEY}" \
    -H "Content-Type: application/json" \
    --data-binary @-
}

extract_findings_json() {
  local content
  content=$(jq -r '.choices[0].message.content // ""')
  if [ -z "$content" ]; then
    echo "review: the model returned no content" >&2
    exit 1
  fi
  printf '%s\n' "$content" | jq -e '.violations | arrays' > /dev/null
  printf '%s\n' "$content"
}

main() {
  local mode="$1" system_prompt diff_text
  system_prompt=$(scripts/review/build-review-prompt.sh "$mode")
  diff_text=$(scripts/review/read-pull-request-diff.sh)
  build_request_body "$(select_model "$mode")" "$system_prompt" "$diff_text" | send_request | extract_findings_json
}

main "$@"
