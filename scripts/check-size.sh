#!/usr/bin/env bash
# Enforce the 1 MiB compiled-binary budget.
# Usage: ./scripts/check-size.sh [path-to-binary]   (default: target/release/ff)
set -euo pipefail

LIMIT=1048576
BINARY="${1:-target/release/ff}"

if [ ! -f "$BINARY" ]; then
  echo "error: binary not found: $BINARY" >&2
  echo "usage: $0 [path-to-binary]" >&2
  exit 1
fi

size=$(wc -c <"$BINARY" | tr -d ' ')
headroom=$((LIMIT - size))

printf 'binary:   %s\n' "$BINARY"
printf 'size:     %s bytes (%s KiB)\n' "$size" "$((size / 1024))"
printf 'budget:   %s bytes (%s KiB)\n' "$LIMIT" "$((LIMIT / 1024))"

if [ "$size" -gt "$LIMIT" ]; then
  printf 'status:   FAIL - over budget by %s bytes\n' "$((size - LIMIT))" >&2
  exit 1
fi

printf 'headroom: %s bytes (%s%% of budget used)\n' "$headroom" "$((size * 100 / LIMIT))"
