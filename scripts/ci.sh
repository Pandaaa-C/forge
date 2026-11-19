#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> fmt"
cargo fmt --all --check

echo "==> clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> test"
cargo test --workspace

echo "==> zero external crates"
bash scripts/check-no-deps.sh

echo "All checks passed."
