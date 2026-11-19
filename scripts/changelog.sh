#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: changelog.sh <version> [--write]}"
mode="${2:-}"

last_tag=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)
range="${last_tag:+$last_tag..}HEAD"

section() {
  local title="$1" pattern="$2" lines
  lines=$(
    git log "$range" --no-merges --format=%s \
      | grep -E "$pattern" \
      | sed -E 's/^[a-z]+(\(([^)]*)\))?!?: (.*)$/- **\2:** \3/; s/^- \*\*:\*\* /- /' \
      || true
  )
  if [[ -n "$lines" ]]; then
    printf '### %s\n\n%s' "$title" "$lines"
  fi
}

parts=()
for spec in \
  "Breaking changes|^[a-z]+(\([^)]*\))?!:" \
  "Features|^feat(\([^)]*\))?:" \
  "Bug fixes|^fix(\([^)]*\))?:" \
  "Performance|^perf(\([^)]*\))?:"; do
  part=$(section "${spec%%|*}" "${spec#*|}")
  if [[ -n "$part" ]]; then
    parts+=("$part")
  fi
done

if [[ ${#parts[@]} -eq 0 ]]; then
  sections="No user-facing changes."
else
  sections="${parts[0]}"
  for part in "${parts[@]:1}"; do
    sections+=$'\n\n'"$part"
  done
fi

if [[ "$mode" == --write ]]; then
  tmp=$(mktemp)
  {
    head -n 4 CHANGELOG.md
    printf '## [%s] - %s\n\n%s\n\n' "$version" "$(date -u +%Y-%m-%d)" "$sections"
    tail -n +5 CHANGELOG.md
  } > "$tmp"
  mv "$tmp" CHANGELOG.md
else
  printf '%s\n' "$sections"
fi
