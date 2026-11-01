#!/bin/sh
stage=$1
shift
start=$(date +%s.%N)
"$@"
status=$?
end=$(date +%s.%N)
awk -v stage="$stage" -v start="$start" -v end="$end" 'BEGIN { printf "%s = %.2f\n", stage, end - start }' >> "$MEASURED"
exit "$status"
