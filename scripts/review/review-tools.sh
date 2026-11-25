#!/usr/bin/env bash

readonly MAXIMUM_TOOL_OUTPUT_BYTES=30000
readonly MAXIMUM_GREP_MATCHES=100
readonly MAXIMUM_LISTED_FILES=200
readonly MAXIMUM_READ_LINE=1000000
readonly MAXIMUM_FETCH_BYTES=2000000
readonly MAXIMUM_FETCH_REDIRECTS=0
readonly FETCH_TIMEOUT_SECONDS=20
readonly ALLOWED_FETCH_HOSTS_FILE="${BASH_SOURCE[0]%/*}/allowed-fetch-hosts.txt"

limit_output() {
  local text
  text=$(head -c "$((MAXIMUM_TOOL_OUTPUT_BYTES + 1))")
  if [ "${#text}" -gt "${MAXIMUM_TOOL_OUTPUT_BYTES}" ]; then
    printf '%s\n[output truncated after %s bytes]\n' "${text:0:MAXIMUM_TOOL_OUTPUT_BYTES}" "${MAXIMUM_TOOL_OUTPUT_BYTES}"
  else
    printf '%s\n' "${text}"
  fi
}

limit_lines() {
  local maximum_lines="$1" lines
  lines=$(head -n "$((maximum_lines + 1))")
  if [ "$(wc -l <<<"${lines}")" -gt "${maximum_lines}" ]; then
    head -n "${maximum_lines}" <<<"${lines}"
    echo "[output truncated after ${maximum_lines} lines]"
  else
    printf '%s\n' "${lines}"
  fi
}

resolve_inside_workspace() {
  local resolved
  resolved=$(realpath -- "$1" 2>&1) || { echo "error: cannot resolve path: ${resolved}"; return 1; }
  [[ "${resolved}" == "${PWD}" || "${resolved}" == "${PWD}"/* ]] || { echo "error: path is outside the repository"; return 1; }
  printf '%s\n' "${resolved}"
}

read_repository_file() {
  local arguments="$1" path start_line end_line resolved
  path=$(jq -r '.path' <<<"${arguments}")
  start_line=$(jq -r '.start_line // 1' <<<"${arguments}")
  end_line=$(jq -r --argjson fallback "${MAXIMUM_READ_LINE}" '.end_line // $fallback' <<<"${arguments}")
  resolved=$(resolve_inside_workspace "${path}") || { echo "${resolved}"; return 0; }
  [[ -f "${resolved}" ]] || { echo "error: not a file"; return 0; }
  awk -v from="${start_line}" -v to="${end_line}" 'NR >= from && NR <= to { printf "%d\t%s\n", NR, $0 }' "${resolved}" | limit_output
}

convert_glob_to_pattern() {
  local double_star_marker='@@DOUBLE_STAR@@'
  printf '^%s$\n' "$(sed -E "s/[][+^\${}()|.\\\\]/\\\\&/g; s/\*\*/${double_star_marker}/g; s/\*/[^\/]*/g; s/\?/[^\/]/g; s/${double_star_marker}/.*/g" <<<"$1")"
}

list_repository_files() {
  local pattern
  pattern=$(jq -r '.glob' <<<"$1")
  git ls-files --cached --others --exclude-standard | grep -E "$(convert_glob_to_pattern "${pattern}")" | limit_lines "${MAXIMUM_LISTED_FILES}"
}

search_repository() {
  local arguments="$1" pattern path resolved
  pattern=$(jq -r '.pattern' <<<"${arguments}")
  path=$(jq -r '.path // "."' <<<"${arguments}")
  resolved=$(resolve_inside_workspace "${path}") || { echo "${resolved}"; return 0; }
  local pattern_check status=0
  pattern_check=$(grep -E -- "${pattern}" /dev/null 2>&1) || status=$?
  if [ "${status}" -gt 1 ]; then
    echo "error: grep failed: ${pattern_check}"
    return 0
  fi
  local workspace_prefix="${PWD}/"
  grep -rnIE --exclude-dir=.git --exclude-dir=target -- "${pattern}" "${resolved}" 2>&1 \
    | head -n "$((MAXIMUM_GREP_MATCHES + 1))" | sed "s#${workspace_prefix}##" | limit_lines "${MAXIMUM_GREP_MATCHES}" | limit_output
}

fetch_documentation_page() {
  local url host
  url=$(jq -r '.url' <<<"$1")
  [[ "${url}" =~ ^https://([^/:]+)(/|$) ]] || { echo "error: only https urls are allowed"; return 0; }
  host="${BASH_REMATCH[1]}"
  grep -qxF -- "${host}" "${ALLOWED_FETCH_HOSTS_FILE}" || { echo "error: host ${host} is not an allowed documentation host"; return 0; }
  curl -fsSL --proto '=https' --max-redirs "${MAXIMUM_FETCH_REDIRECTS}" --max-time "${FETCH_TIMEOUT_SECONDS}" --max-filesize "${MAXIMUM_FETCH_BYTES}" "${url}" 2>&1 \
    | sed 's/<script[^>]*>.*<\/script>//g; s/<[^>]*>/ /g; s/&nbsp;/ /g; s/&quot;/"/g; s/&amp;/\&/g' | tr -s ' \n' | limit_output
}

run_tool() {
  case "$1" in
    read_file) read_repository_file "$2" ;;
    list_files) list_repository_files "$2" ;;
    grep) search_repository "$2" ;;
    fetch_url) fetch_documentation_page "$2" ;;
    *) echo "error: unknown tool $1" ;;
  esac
}
