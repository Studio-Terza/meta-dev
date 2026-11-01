#!/bin/bash
set -euo pipefail
cargo build -q --bin reach --bin lint
mkdir -p "$TOOLS"
cp target/debug/reach target/debug/lint "$TOOLS"
