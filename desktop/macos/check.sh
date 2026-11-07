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

# Click: activating the link "How it works" opens its page (same handler a pointer click calls).
clicked="$("$out/aqhost" "$json" --click "How it works")"
if [ "$(printf '%s\n' "$clicked" | head -1)" != "$(printf 'AXStaticText\tHow it works\tnot-focusable')" ] \
  || printf '%s\n' "$clicked" | grep -q "One language for coding agents"; then
  echo "FAIL: click did not open the page: $(printf '%s\n' "$clicked" | head -1)"; exit 1
fi
if ! printf '%s\n' "$clicked" | grep -q "^AXButton	Back home	focusable$"; then
  echo "FAIL: the page opened by the click has no 'Back home' link"; exit 1
fi
if "$out/aqhost" "$json" --click "No such link" >/dev/null 2>&1; then
  echo "FAIL: a click on a missing link should fail"; exit 1
fi
# Unknown route: the host refuses it with exit 1 (not 0, not a crash).
rc=0; "$out/aqhost" "$json" --route /nope >/dev/null 2>&1 || rc=$?
if [ "$rc" != "1" ]; then
  echo "FAIL: unknown route should exit 1, got $rc"; exit 1
fi
echo "ok: unknown route exits 1"
echo "ok: click opens the page for 'How it works' (activation via target-action, not an OS pointer event)"

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
