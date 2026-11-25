#!/usr/bin/env bash

declare -rx SHORT_SHA_LENGTH=7

review_result_path() {
  echo "${RUNNER_TEMP:-/tmp}/review-result-$1"
}
