#!/usr/bin/env python3
"""AQELITH desktop host for Linux (docs/platforms/DESKTOP_LINUX.md).

Draws the JSON produced by `aqelith view <file.aq>`: one window, the page title as the window title,
headings, paragraphs and links. A link switches to the page with that route. The language knows
nothing about GTK.

usage: aqhost.py <view.json> [--route /path] [--width N] [--snapshot out.png] [--ax-dump] [--click LABEL]
  --snapshot saves the window as a PNG and exits (needs an X display, e.g. under xvfb-run).
  --ax-dump prints one line per control: role, label, focusable. Needs no display.
  --click LABEL activates that link, then prints the accessibility lines of the page it opens.
"""
import json
import subprocess
import sys

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import GLib, Gtk, Pango  # noqa: E402


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)["pages"]


def page_for(pages, route):
    for p in pages:
        if route is None or p["route"] == route:
            return p
    return None


class Host:
    def __init__(self, pages, start, width):
        self.pages = pages
        self.width = width
        self.window = Gtk.Window(title=start["title"])
        self.window.set_default_size(width, 200)
        self.box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.box.set_margin_top(24)
        self.box.set_margin_bottom(24)
        self.box.set_margin_start(24)
        self.box.set_margin_end(24)
        self.window.add(self.box)
        self.window.connect("destroy", Gtk.main_quit)
        self.show(start)

    def show(self, page):
        self.window.set_title(page["title"])
        for child in self.box.get_children():
            self.box.remove(child)
        for item in page["items"]:
            kind = item["kind"]
            if kind == "heading":
                label = Gtk.Label(label=item["text"])
                label.set_xalign(0)
                label.set_line_wrap(True)
                size = 22 if item["level"] == 1 else 16
                label.set_markup(f'<span size="{size * 1024}" weight="bold">{GLib.markup_escape_text(item["text"])}</span>')
                label.get_accessible().set_name(item["text"])
                self.box.pack_start(label, False, False, 0)
            elif kind == "para":
                label = Gtk.Label(label=item["text"])
                label.set_xalign(0)
                label.set_line_wrap(True)
                label.set_line_wrap_mode(Pango.WrapMode.WORD)
                label.get_accessible().set_name(item["text"])
                self.box.pack_start(label, False, False, 0)
            elif kind == "link":
                button = Gtk.LinkButton.new_with_label(item["href"], item["label"])
                button.set_halign(Gtk.Align.START)
                button.connect("clicked", lambda b: self.on_link(b.get_uri()))
                self.box.pack_start(button, False, False, 0)
        self.box.show_all()

    def on_link(self, href):
        # The link's href is a route of this site: switch page instead of opening a browser.
        target = page_for(self.pages, href)
        if target is not None:
            self.show(target)

    def click(self, label):
        """Activates the link with this label, as a pointer click would (same handler)."""
        for child in self.box.get_children():
            if isinstance(child, Gtk.LinkButton) and child.get_label() == label:
                child.emit("clicked")
                return True
        return False

    def ax_lines(self):
        lines = []
        for child in self.box.get_children():
            if isinstance(child, Gtk.LinkButton):
                lines.append(f"link\t{child.get_label()}\tfocusable")
            elif isinstance(child, Gtk.Label):
                lines.append(f"text\t{child.get_text()}\tnot-focusable")
        return lines


def main(argv):
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    route = None
    snapshot = None
    width = 640
    ax_dump = False
    click = None
    i = 2
    while i < len(argv):
        if argv[i] == "--route" and i + 1 < len(argv):
            route = argv[i + 1]
            i += 2
        elif argv[i] == "--snapshot" and i + 1 < len(argv):
            snapshot = argv[i + 1]
            i += 2
        elif argv[i] == "--width" and i + 1 < len(argv):
            width = int(argv[i + 1])
            if width < 200:
                print("aqhost: --width needs a number of at least 200", file=sys.stderr)
                return 2
            i += 2
        elif argv[i] == "--ax-dump":
            ax_dump = True
            i += 1
        elif argv[i] == "--click" and i + 1 < len(argv):
            click = argv[i + 1]
            i += 2
        else:
            print(f"aqhost: unknown argument {argv[i]}", file=sys.stderr)
            return 2
    pages = load(argv[1])
    start = page_for(pages, route)
    if start is None:
        print("aqhost: no page for the requested route", file=sys.stderr)
        return 1
    if click is not None:
        host = Host(pages, start, width)
        if not host.click(click):
            print(f"aqhost: no link labelled {click!r} on this page", file=sys.stderr)
            return 1
        for line in host.ax_lines():
            print(line)
        return 0
    if ax_dump:
        host = Host(pages, start, width)
        for line in host.ax_lines():
            print(line)
        return 0
    host = Host(pages, start, width)
    host.window.show_all()
    if snapshot:
        def take():
            while Gtk.events_pending():
                Gtk.main_iteration()
            subprocess.run(["import", "-window", "root", snapshot], check=True)
            Gtk.main_quit()
            return False
        GLib.timeout_add(700, take)
    Gtk.main()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
