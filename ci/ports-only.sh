#!/usr/bin/env bash
set -euo pipefail

crates="${1:-crates}"
allowed="$crates/meta-ports/src/real/"
pattern='std::(process|env)\b|std::\{[^}]*\b(process|env)\b'

hits="$(grep -rnE "$pattern" --include='*.rs' "$crates" | grep -v "^$allowed" || true)"

if [ -n "$hits" ]; then
  echo "std::process and std::env belong only in $allowed:"
  echo "$hits"
  exit 1
fi
echo "ports-only: std::process and std::env appear only in $allowed"
