#!/bin/bash
set -euo pipefail
metadata=$RUNNER_TEMP/metadata.json
cargo metadata --no-deps --format-version 1 > "$metadata"
if [ "$GITHUB_EVENT_NAME" = pull_request ]; then
  git diff --name-only --no-renames HEAD^1 HEAD > "$RUNNER_TEMP/changed.txt"
  mapfile -t changed < "$RUNNER_TEMP/changed.txt"
  cargo run -q --bin reach -- affected "$metadata" "${changed[@]}"
else
  echo "CARGO_SCOPE=--workspace"
fi
