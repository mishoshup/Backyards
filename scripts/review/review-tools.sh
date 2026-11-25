#!/usr/bin/env bash

readonly MAXIMUM_TOOL_OUTPUT_BYTES=30000
readonly MAXIMUM_GREP_MATCHES=100
readonly FETCH_TIMEOUT_SECONDS=20
readonly ALLOWED_FETCH_HOSTS='^(api-docs\.deepseek\.com|docs\.rs|doc\.rust-lang\.org|docs\.github\.com|crates\.io|docs\.opencode\.ai|opencode\.ai|rust-lang\.github\.io|clap\.rs|serde\.rs)$'

limit_output() {
  head -c "${MAXIMUM_TOOL_OUTPUT_BYTES}"
}

resolve_inside_workspace() {
  local resolved
  resolved=$(realpath -- "$1" 2>/dev/null) || return 1
  [[ "${resolved}" == "${PWD}" || "${resolved}" == "${PWD}"/* ]] || return 1
  printf '%s\n' "${resolved}"
}

tool_read_file() {
  local arguments="$1" path start_line end_line resolved
  path=$(jq -r '.path' <<<"${arguments}")
  start_line=$(jq -r '.start_line // 1' <<<"${arguments}")
  end_line=$(jq -r '.end_line // 100000' <<<"${arguments}")
  resolved=$(resolve_inside_workspace "${path}") || { echo "error: path is outside the repository or missing"; return 0; }
  [[ -f "${resolved}" ]] || { echo "error: not a file"; return 0; }
  awk -v from="${start_line}" -v to="${end_line}" 'NR >= from && NR <= to { printf "%d\t%s\n", NR, $0 }' "${resolved}" | limit_output
}

tool_list_files() {
  local pattern
  pattern=$(jq -r '.glob' <<<"$1")
  git ls-files --cached --others --exclude-standard | grep -E "$(sed 's/[.]/\\./g; s/\*/.*/g' <<<"${pattern}")" | head -200
}

tool_grep() {
  local arguments="$1" pattern path resolved
  pattern=$(jq -r '.pattern' <<<"${arguments}")
  path=$(jq -r '.path // "."' <<<"${arguments}")
  resolved=$(resolve_inside_workspace "${path}") || { echo "error: path is outside the repository or missing"; return 0; }
  grep -rnIE --exclude-dir=.git --exclude-dir=target -- "${pattern}" "${resolved}" 2>/dev/null \
    | sed "s#^${PWD}/##" | head -n "${MAXIMUM_GREP_MATCHES}" | limit_output
}

tool_fetch_url() {
  local url host
  url=$(jq -r '.url' <<<"$1")
  [[ "${url}" =~ ^https://([^/:]+)(/|$) ]] || { echo "error: only https urls are allowed"; return 0; }
  host="${BASH_REMATCH[1]}"
  [[ "${host}" =~ ${ALLOWED_FETCH_HOSTS} ]] || { echo "error: host ${host} is not an allowed documentation host"; return 0; }
  curl -fsSL --proto '=https' --max-redirs 3 --max-time "${FETCH_TIMEOUT_SECONDS}" --max-filesize 2000000 "${url}" 2>&1 \
    | sed 's/<script[^>]*>.*<\/script>//g; s/<[^>]*>/ /g; s/&nbsp;/ /g; s/&quot;/"/g; s/&amp;/\&/g' | tr -s ' \n' | limit_output
}

run_tool() {
  case "$1" in
    read_file) tool_read_file "$2" ;;
    list_files) tool_list_files "$2" ;;
    grep) tool_grep "$2" ;;
    fetch_url) tool_fetch_url "$2" ;;
    *) echo "error: unknown tool $1" ;;
  esac
}
