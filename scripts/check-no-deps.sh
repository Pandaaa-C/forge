#!/usr/bin/env bash
set -euo pipefail

root="$(pwd)"
external=$(
  cargo tree --workspace --edges normal,build,dev --prefix none --no-dedupe --format '{p}' \
    | grep -v '^$' \
    | grep -vF "($root" \
    | sort -u \
    || true
)

if [[ -n "$external" ]]; then
  echo "External crates found. forge must only depend on its own crates:"
  echo "$external"
  exit 1
fi

echo "OK: no external crates."
