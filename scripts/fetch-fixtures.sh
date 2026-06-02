#!/usr/bin/env bash
# Fetch a curated set of public Anchor programs into tests/fixtures/public/.
# These are used as integration samples — sentinel scans them and the
# integration test confirms it doesn't panic and produces a reasonable
# number of findings on known-vulnerable code (Sealevel Attacks) and
# zero findings on clean modern code (anchor/examples counter).
#
# Usage:  ./scripts/fetch-fixtures.sh
# Idempotent: re-running just refreshes each repo.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/tests/fixtures/public"
mkdir -p "$DEST"

declare -a REPOS=(
  "anchor-counter|https://github.com/anchor-book/anchor-counter.git"
  "anchor-examples|https://github.com/coral-xyz/anchor-examples.git"
  "sealevel-attacks|https://github.com/coral-xyz/sealevel-attacks.git"
  "anchor-zero-copy|https://github.com/anchor-lang/anchor-zero-copy.git"
)

for entry in "${REPOS[@]}"; do
  name="${entry%%|*}"
  url="${entry##*|}"
  target="$DEST/$name"
  if [[ -d "$target/.git" ]]; then
    echo "↻ refreshing $name"
    git -C "$target" pull --depth 1 --ff-only || true
  else
    echo "↓ cloning $name from $url"
    git clone --depth 1 "$url" "$target"
  fi
done

echo
echo "fixtures available under: $DEST"
ls -1 "$DEST"
