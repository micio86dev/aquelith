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
# Accessibility, per route: every link of the page is a focusable button with its label, and every
# text item has a label.
json="$out/landing.json"
for route in / /how /start; do
  links=$(python3 -c 'import json,sys
pages=json.load(open(sys.argv[1]))["pages"]
page=[p for p in pages if p["route"]==sys.argv[2]][0]
print(sum(1 for i in page["items"] if i["kind"]=="link"))' "$json" "$route")
  dump=$("$out/aqhost" "$json" --route "$route" --ax-dump)
  buttons=$(printf '%s\n' "$dump" | awk -F'\t' '$1=="AXButton" && $2!="" && $3=="focusable"' | wc -l | tr -d ' ')
  unlabeled=$(printf '%s\n' "$dump" | awk -F'\t' '$2==""' | wc -l | tr -d ' ')
  if [ "$links" != "$buttons" ] || [ "$unlabeled" != "0" ]; then
    echo "FAIL: accessibility $route: $links links, $buttons labelled focusable buttons, $unlabeled unlabelled"; exit 1
  fi
  echo "ok: accessibility $route ($links links, all focusable and labelled)"
done

# Width: the same page is taller in a narrow window, because its text wraps.
"$out/aqhost" "$json" --width 300 --snapshot "$out/narrow.png"
"$out/aqhost" "$json" --width 900 --snapshot "$out/wide.png"
height() { python3 -c 'import struct,sys; print(struct.unpack(">I", open(sys.argv[1],"rb").read()[20:24])[0])' "$1"; }
hn=$(height "$out/narrow.png"); hw=$(height "$out/wide.png")
if [ "$hn" -le "$hw" ]; then
  echo "FAIL: width: narrow height $hn is not taller than wide height $hw"; exit 1
fi
echo "ok: width (narrow ${hn}px tall, wide ${hw}px tall)"

echo "macOS host checks passed"
