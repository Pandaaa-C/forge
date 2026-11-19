#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: set-version.sh <version>}"

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "Invalid version '$version'." >&2
  exit 1
fi

awk -v v="$version" '
  !done && /^version = "/ { print "version = \"" v "\""; done = 1; next }
  /^forge-[a-z-]+ = \{/ { sub(/version = "[^"]*"/, "version = \"" v "\"") }
  { print }
' Cargo.toml > Cargo.toml.tmp
mv Cargo.toml.tmp Cargo.toml

cargo update --workspace --quiet

echo "Version set to $version"
