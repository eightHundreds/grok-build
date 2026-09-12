#!/usr/bin/env bash
# Resolve this fork's published version from the upstream X.Y.Z crate version.
#
# Rule: never invent a different major.minor.patch. Each fork cut is
#   <upstream>-fork.<n>
# e.g. upstream 1.0.24 → 1.0.24-fork.1, 1.0.24-fork.2, …
#
# Inputs:
#   crates/codegen/xai-grok-version/Cargo.toml  (must be plain X.Y.Z)
#   GITHUB_REF  — if refs/tags/vX.Y.Z-fork.N, validate and reuse
#   FORK_EXISTING_TAGS — optional newline-separated tags (tests / offline)
#   GH_TOKEN / GITHUB_TOKEN + gh — used to list existing releases
#
# Writes KEY=VALUE lines to stdout (and to $GITHUB_OUTPUT when set).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
crate=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$root/crates/codegen/xai-grok-version/Cargo.toml" | head -1)
if [[ ! "$crate" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "upstream crate version must be plain X.Y.Z (no suffix): got '${crate:-<empty>}'" >&2
  exit 1
fi

crate_re="${crate//./\\.}"

emit() {
  printf '%s\n' "$1"
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf '%s\n' "$1" >> "$GITHUB_OUTPUT"
  fi
}

list_existing_tags() {
  if [[ -n "${FORK_EXISTING_TAGS:-}" ]]; then
    printf '%s\n' "$FORK_EXISTING_TAGS"
    return
  fi
  if command -v gh >/dev/null 2>&1 && [[ -n "${GH_TOKEN:-}${GITHUB_TOKEN:-}" ]]; then
    local repo="${GITHUB_REPOSITORY:-eightHundreds/grok-build}"
    gh release list --repo "$repo" --limit 500 --json tagName --jq '.[].tagName' 2>/dev/null || true
  fi
}

max_fork_n() {
  local max=0 tag n
  while IFS= read -r tag; do
    [[ -z "$tag" ]] && continue
    tag="${tag#v}"
    if [[ "$tag" =~ ^${crate_re}-fork\.([0-9]+)$ ]]; then
      n="${BASH_REMATCH[1]}"
      # strip leading zeros so 08 is 8; reject empty
      n=$((10#$n))
      if (( n > max )); then
        max=$n
      fi
    fi
  done < <(list_existing_tags)
  printf '%s\n' "$max"
}

if [[ "${GITHUB_REF:-}" == refs/tags/v* ]]; then
  version="${GITHUB_REF_NAME#v}"
  if [[ ! "$version" =~ ^${crate_re}-fork\.[1-9][0-9]*$ ]]; then
    echo "tag v${version} must be v${crate}-fork.N (N>=1) so the X.Y.Z stays ${crate}" >&2
    exit 1
  fi
else
  next=$(( $(max_fork_n) + 1 ))
  version="${crate}-fork.${next}"
fi

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+-fork\.[1-9][0-9]*$ ]]; then
  echo "invalid fork version: $version" >&2
  exit 1
fi

fork_n="${version##*-fork.}"
title="Grok Build ${crate} Fork ${fork_n}"

emit "upstream=${crate}"
emit "version=${version}"
emit "tag=v${version}"
emit "fork_n=${fork_n}"
emit "title=${title}"
echo "Resolved fork version ${version} (upstream=${crate} title=${title})" >&2
