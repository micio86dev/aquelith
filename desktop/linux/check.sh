#!/bin/bash
# Verifies the Linux desktop host in a container: builds the image, exports the landing pages from
# the compiler on this machine, then per route checks the accessibility tree and renders a PNG under a
# virtual X display. Run from anywhere: bash desktop/linux/check.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
docker build -q --platform linux/arm64 -t aqelith-linux-desktop "$here" >/dev/null
(cd "$repo/compiler" && cargo build -q --bins)
"$repo/compiler/target/debug/aqelith" view "$repo/web/examples/landing/landing.aq" > "$out/landing.json"
run() {
  docker run --rm --platform linux/arm64 --network=none -e NO_AT_BRIDGE=1 -v "$out:/data:rw" \
    --entrypoint sh aqelith-linux-desktop -c "$1"
}
for route in / /how /start; do
  name="$(echo "$route" | tr '/' '_')"
  links=$(python3 -c 'import json,sys
pages=json.load(open(sys.argv[1]))["pages"]
page=[p for p in pages if p["route"]==sys.argv[2]][0]
print(sum(1 for i in page["items"] if i["kind"]=="link"))' "$out/landing.json" "$route")
  dump=$(run "timeout 60 xvfb-run -a python3 /app/aqhost.py /data/landing.json --route '$route' --ax-dump")
  buttons=$(printf '%s\n' "$dump" | awk -F'\t' '$1=="link" && $2!="" && $3=="focusable"' | wc -l | tr -d ' ')
  if [ "$links" != "$buttons" ]; then
    echo "FAIL: accessibility $route: $links links, $buttons focusable labelled"; exit 1
  fi
  echo "ok: accessibility $route ($links links, focusable and labelled)"
  run "timeout 90 xvfb-run -a -s '-screen 0 800x600x24' python3 /app/aqhost.py /data/landing.json --route '$route' --snapshot /data/$name.png" >/dev/null
  size=$(wc -c < "$out/$name.png" | tr -d ' ')
  if [ "$size" -lt 4000 ]; then
    echo "FAIL: $route rendered only $size bytes"; exit 1
  fi
  echo "ok: $route rendered ($size bytes)"
done
# Click: activating the link "How it works" opens its page (same handler as a pointer click).
clicked=$(run "timeout 60 xvfb-run -a python3 /app/aqhost.py /data/landing.json --click 'How it works' 2>/dev/null")
first=$(printf '%s\n' "$clicked" | head -1)
if [ "$first" != "$(printf 'text\tHow it works\tnot-focusable')" ]; then
  echo "FAIL: click did not open the page: first line was '$first'"; exit 1
fi
echo "ok: click opens the page for 'How it works'"

echo "Linux desktop host checks passed (arm64 Linux container under Docker; not a native Linux desktop)"
