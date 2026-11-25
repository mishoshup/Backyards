#!/usr/bin/env bash

readonly DEEPSEEK_CHAT_URL="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}/chat/completions"

select_review_model() {
  case "$1" in
    inline) echo "${REVIEW_MODEL_INLINE:-deepseek-flash}" ;;
    holistic) echo "${REVIEW_MODEL_HOLISTIC:-deepseek-flash}" ;;
  esac
}

post_to_deepseek() {
  curl "$@" "${DEEPSEEK_CHAT_URL}" \
    -H "Authorization: Bearer ${DEEPSEEK_API_KEY}" \
    -H "Content-Type: application/json" \
    --data-binary @-
}
