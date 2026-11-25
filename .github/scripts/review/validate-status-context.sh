#!/usr/bin/env bash

readonly STATUS_CONTEXT_PATTERN='^[a-z0-9]([a-z0-9/_-]*[a-z0-9])?$'

validate_status_context() {
  [[ "$1" =~ ${STATUS_CONTEXT_PATTERN} ]] || { echo "status context must be lowercase with / _ - separators" >&2; exit 1; }
}
