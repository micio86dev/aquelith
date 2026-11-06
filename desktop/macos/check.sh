#!/bin/bash
# Verifies the macOS host on this machine: builds the host, exports the landing pages, renders each
# route to a PNG and checks that every PNG is non-trivial. Run from anywhere: bash desktop/macos/check.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
swiftc -O "$here/aqhost.swift" -o "$out/aqhost"
(cd "$repo/compiler" && cargo build -q --bins)
"$repo/compiler/target/debug/aqelith" view "$repo/web/examples/landing/landing.aq" > "$out/landing.json"
for route in / /how /start; do
  name="$(echo "$route" | tr '/' '_')"
  "$out/aqhost" "$out/landing.json" --route "$route" --snapshot "$out/$name.png"
  size=$(wc -c < "$out/$name.png" | tr -d ' ')
  if [ "$size" -lt 4000 ]; then
    echo "FAIL: $route rendered only $size bytes"; exit 1
  fi
  echo "ok: $route rendered ($size bytes)"
done
echo "macOS host checks passed"
