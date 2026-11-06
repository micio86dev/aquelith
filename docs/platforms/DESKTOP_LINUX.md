# Desktop target: Linux (GTK 3, in a container)

Status: **started. The host draws a checked module's pages under a virtual display; the checks pass in a Docker container.** Not a native Linux desktop yet.

## What exists

- `aqelith view <file.aq>` exports the pages as JSON (shared with the macOS target, [DESKTOP_MACOS.md](DESKTOP_MACOS.md)).
- `desktop/linux/aqhost.py`: a GTK 3 host that draws the pages: title, headings, paragraphs and links; a link switches to its route.
- `desktop/linux/Dockerfile`: a Debian bookworm image with GTK 3, Xvfb and ImageMagick (about 0.5 GB of packages).
- `desktop/linux/check.sh`: builds the image and checks, for each route of the landing site, that every link is a focusable labelled control, and that a PNG of the window is rendered under Xvfb.

## Verified (2026-11-06)

- Route `/`, `/how`, `/start`: accessibility (2, 1, 1 links, focusable and labelled) and a rendered PNG, inspected by eye for `/`.
- The image runs offline (`--network=none`) on an arm64 host.

## Not verified

- A native Linux desktop, on a real display. The run is inside a container with a virtual X server.
- Keyboard focus order, window resizing, the click path, and assistive technology (AT-SPI). AT-SPI is disabled in the check (`NO_AT_BRIDGE=1`), so the accessibility check is the host's own tree, not the platform's.
- x86_64 Linux: this image is arm64. The emulated x86_64 run covered the server target, not this host.

## Note on the entrypoint

`docker run` with the image's default entrypoint produced no output in this environment; the working form is `--entrypoint sh ... -c 'xvfb-run -a python3 /app/aqhost.py ...'`, which `check.sh` uses. The cause was not identified.
