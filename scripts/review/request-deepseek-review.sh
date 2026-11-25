#!/usr/bin/env bash
set -euo pipefail

base_url="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}"
readonly request_timeout_seconds=600
readonly heartbeat_interval_seconds=15

select_model() {
  case "$1" in
    inline) echo "${REVIEW_MODEL_INLINE:-deepseek-flash}" ;;
    holistic) echo "${REVIEW_MODEL_HOLISTIC:-deepseek-flash}" ;;
  esac
}

build_request_body() {
  local model="$1" system_prompt="$2" diff_text="$3"
  jq -n --arg model "$model" --arg system "$system_prompt" --arg diff "$diff_text" '{
    model: $model,
    stream: true,
    response_format: {type: "json_object"},
    messages: [
      {role: "system", content: $system},
      {role: "user", content: ("Review this diff and answer in JSON.\n\n" + $diff)}
    ]
  }'
}

send_streaming_request() {
  curl -fsSN --max-time "${request_timeout_seconds}" "${base_url}/chat/completions" \
    -H "Authorization: Bearer ${DEEPSEEK_API_KEY}" \
    -H "Content-Type: application/json" \
    --data-binary @-
}

tag_stream_events() {
  sed -u -n 's/^data: //p' | jq --unbuffered -Rr '
    select(. != "[DONE]") | fromjson | .choices[0].delta
    | if (.reasoning_content // "") != "" then "R"
      elif (.content // "") != "" then "C" + (.content | @json)
      else empty end'
}

report_progress() {
  local mode="$1" started_at="$2" reasoning_events="$3"
  echo "review: ${mode} thinking, $((SECONDS - started_at))s elapsed, ${reasoning_events} reasoning chunks" >&2
}

collect_stream() {
  local mode="$1" content_file="$2" started_at=$SECONDS last_report=$SECONDS reasoning_events=0 event
  while IFS= read -r event; do
    if [[ "${event}" == R ]]; then
      reasoning_events=$((reasoning_events + 1))
    else
      jq -j . <<<"${event:1}" >> "${content_file}"
    fi
    if (( SECONDS - last_report >= heartbeat_interval_seconds )); then
      report_progress "${mode}" "${started_at}" "${reasoning_events}"
      last_report=$SECONDS
    fi
  done
  echo "review: ${mode} answer received after $((SECONDS - started_at))s" >&2
}

validate_findings() {
  local content_file="$1"
  if [[ ! -s "${content_file}" ]]; then
    echo "review: the model returned no content" >&2
    exit 1
  fi
  jq -e '.violations | arrays' "${content_file}" > /dev/null
  cat "${content_file}"
}

main() {
  local mode="$1" system_prompt diff_text content_file
  system_prompt=$(scripts/review/build-review-prompt.sh "$mode")
  diff_text=$(scripts/review/read-pull-request-diff.sh)
  content_file=$(mktemp)
  echo "review: ${mode} request started with $(select_model "$mode")" >&2
  build_request_body "$(select_model "$mode")" "$system_prompt" "$diff_text" \
    | send_streaming_request | tag_stream_events | collect_stream "$mode" "$content_file"
  validate_findings "$content_file"
}

main "$@"
