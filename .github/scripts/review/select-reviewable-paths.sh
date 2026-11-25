#!/usr/bin/env bash
set -euo pipefail

readonly changed_files_path="${1:?changed files path required}"
readonly reviewable_pattern='(\.rs$|(^|/)Cargo\.(toml|lock)$|^scripts/|^\.github/|^\.agents/|^\.claude/|^(clippy|deny|rustfmt)\.toml$|^rust-toolchain(\.toml)?$|(^|/)AGENTS\.md$)'

main() {
  grep -qE "${reviewable_pattern}" "${changed_files_path}"
}

main "$@"
