#!/usr/bin/env bash
set -euo pipefail

bump="${1:-auto}"
current=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
last_tag=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)

if [[ "$bump" == auto ]]; then
  if [[ -z "$last_tag" ]]; then
    echo "$current"
    exit 0
  fi

  subjects=$(git log "$last_tag..HEAD" --no-merges --format=%s)
  bodies=$(git log "$last_tag..HEAD" --no-merges --format=%b)

  if grep -qE '^[a-z]+(\([^)]*\))?!:' <<<"$subjects" || grep -q '^BREAKING CHANGE' <<<"$bodies"; then
    bump=major
  elif grep -qE '^feat(\([^)]*\))?:' <<<"$subjects"; then
    bump=minor
  elif grep -qE '^(fix|perf)(\([^)]*\))?:' <<<"$subjects"; then
    bump=patch
  else
    echo "Nothing to release: no feat, fix, perf or breaking commits since $last_tag." >&2
    exit 1
  fi

  if [[ "$current" == 0.* ]]; then
    case "$bump" in
      major) bump=minor ;;
      minor) bump=patch ;;
    esac
  fi
fi

IFS=. read -r major minor patch <<<"${current%%-*}"

case "$bump" in
  major) echo "$((major + 1)).0.0" ;;
  minor) echo "$major.$((minor + 1)).0" ;;
  patch) echo "$major.$minor.$((patch + 1))" ;;
  *)
    echo "Unknown bump '$bump'. Use auto, patch, minor or major." >&2
    exit 1
    ;;
esac
