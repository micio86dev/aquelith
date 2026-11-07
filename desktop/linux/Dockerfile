# Linux desktop host for AQELITH (docs/platforms/DESKTOP_LINUX.md): GTK 3 under a virtual X display.
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      python3 python3-gi gir1.2-gtk-3.0 xvfb xauth xdotool imagemagick fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*
COPY aqhost.py /app/aqhost.py
# Run it with: docker run --rm --network=none -v DIR:/data --entrypoint sh IMAGE -c 'xvfb-run -a python3 /app/aqhost.py /data/view.json ...'
