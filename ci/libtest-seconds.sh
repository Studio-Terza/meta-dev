#!/bin/sh
awk -v stage="$1" '/^test result/ { for (i = 1; i <= NF; i++) if ($i == "in") total += $(i + 1) } END { printf "%s = %.2f\n", stage, total }' "$2" >> "$MEASURED"
