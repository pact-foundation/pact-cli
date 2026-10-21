#!/usr/bin/env bash
# Usage: npm-publish-tag.sh <version> <current-latest>
#
# Prints the npm dist-tag to publish <version> under. npm moves `latest` to
# whatever is published last, so a retry or backfill of an older version goes
# to `backfill` and leaves `latest` on the highest version.
set -euo pipefail

version="$1"
latest="${2:-}"

if [[ -z "$latest" ]]; then
  echo latest
  exit 0
fi

IFS=. read -r v1 v2 v3 <<< "$version"
IFS=. read -r l1 l2 l3 <<< "$latest"
for pair in "$v1 $l1" "$v2 $l2" "$v3 $l3"; do
  read -r v l <<< "$pair"
  if (( 10#$v > 10#$l )); then
    echo latest
    exit 0
  fi
  if (( 10#$v < 10#$l )); then
    echo backfill
    exit 0
  fi
done
echo backfill
