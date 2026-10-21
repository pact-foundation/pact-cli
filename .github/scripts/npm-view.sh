#!/usr/bin/env bash
# Usage: npm-view.sh <package-spec> <field>
#
# Prints `npm view <package-spec> <field>`, or nothing when the package or
# version does not exist. Any other registry error fails, so a transient outage
# cannot read as "not published" and let an older version take `latest`.
set -euo pipefail

err="$(mktemp)"
trap 'rm -f "$err"' EXIT

if npm view "$1" "$2" 2> "$err"; then
  exit 0
fi
if grep -q 'code E404' "$err"; then
  exit 0
fi
cat "$err" >&2
exit 1
