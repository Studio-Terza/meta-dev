#!/usr/bin/env bash
set -euo pipefail

crates="${1:-crates}"
allowed="$crates/meta-ports/src/real/"
pattern='std::(process|env)\b|std::\{[^;]*\b(process|env)\b'

hits=""
while IFS= read -r file; do
  case "$file" in
    "$allowed"*) continue ;;
  esac
  if tr '\n' ' ' < "$file" | grep -qE "$pattern"; then
    hits="$hits$file"$'\n'
  fi
done < <(find "$crates" -name '*.rs' | sort)

if [ -n "$hits" ]; then
  echo "std::process and std::env belong only in $allowed:"
  printf '%s' "$hits"
  exit 1
fi
echo "ports-only: std::process and std::env appear only in $allowed"
