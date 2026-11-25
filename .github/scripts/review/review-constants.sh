#!/usr/bin/env bash

declare -rx SHORT_SHA_LENGTH=7
declare -rx RULE_PATTERN='[A-Z]+(-[A-Z]+)*-[0-9]+'

review_result_path() {
  echo "${RUNNER_TEMP:-/tmp}/review-result-$1"
}
