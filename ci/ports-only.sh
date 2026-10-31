#!/usr/bin/env bash
set -euo pipefail

crates="${1:-crates}"
allowed="$crates/meta-ports/src/real/"
# Narrow exception (task-32): the dev-only Scratch temp-dir guard needs std::env::temp_dir and
# std::process::id. This one file, not the testkit crate as a whole.
testkit_scratch="$crates/meta-testkit/src/scratch.rs"
pattern='std::(process|env)\b|std::\{[^;]*\b(process|env)\b'

hits=""
while IFS= read -r file; do
  case "$file" in
    "$allowed"*) continue ;;
    "$testkit_scratch") continue ;;
  esac
  if tr '\n' ' ' < "$file" | grep -qE "$pattern"; then
    hits="$hits$file"$'\n'
  fi
done < <(find "$crates" -name '*.rs' | sort)

if [ -n "$hits" ]; then
  echo "std::process and std::env belong only in $allowed (and $testkit_scratch):"
  printf '%s' "$hits"
  exit 1
fi
echo "ports-only: std::process and std::env appear only in $allowed (and $testkit_scratch)"
