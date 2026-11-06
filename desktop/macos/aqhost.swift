// AQELITH desktop host for macOS (docs/platforms/DESKTOP_MACOS.md). It draws the JSON produced by
// `aqelith view <file.aq>`: one window, the page title as the window title, headings, paragraphs
// and links. A link switches to the page with that route. The language knows nothing of AppKit.
//
// usage: aqhost <view.json> [--route /path] [--snapshot out.png]
//   --snapshot renders the window to a PNG and exits (used by the tests).
import AppKit

struct Item {
    let kind: String
    let text: String
    let level: Int
    let href: String
}

struct Page {
    let name: String
    let route: String
    let title: String
    let items: [Item]
}

func loadPages(_ path: String) -> [Page] {
    guard let data = FileManager.default.contents(atPath: path),
          let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
          let pages = root["pages"] as? [[String: Any]] else {
        fputs("aqhost: cannot read pages from \(path)\n", stderr)
        exit(2)
    }
    return pages.map { p in
        let items = (p["items"] as? [[String: Any]] ?? []).map { i -> Item in
            Item(kind: i["kind"] as? String ?? "",
                 text: (i["text"] as? String) ?? (i["label"] as? String ?? ""),
                 level: i["level"] as? Int ?? 1,
                 href: i["href"] as? String ?? "")
        }
        return Page(name: p["name"] as? String ?? "",
                    route: p["route"] as? String ?? "/",
                    title: p["title"] as? String ?? "",
                    items: items)
    }
}

final class Host: NSObject {
    let pages: [Page]
    let window: NSWindow
    let stack = NSStackView()
    var current: Page

    let width: CGFloat

    init(pages: [Page], start: Page, width: CGFloat) {
        self.pages = pages
        self.current = start
        self.width = width
        self.window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: width, height: 480),
                               styleMask: [.titled, .closable, .resizable],
                               backing: .buffered, defer: false)
        super.init()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12
        stack.edgeInsets = NSEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)
        stack.translatesAutoresizingMaskIntoConstraints = false
        window.contentView = NSView()
        window.contentView?.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: window.contentView!.leadingAnchor),
            stack.trailingAnchor.constraint(lessThanOrEqualTo: window.contentView!.trailingAnchor),
            stack.topAnchor.constraint(equalTo: window.contentView!.topAnchor),
        ])
        show(start)
    }

    func show(_ page: Page) {
        current = page
        window.title = page.title
        for v in stack.arrangedSubviews {
            stack.removeArrangedSubview(v)
            v.removeFromSuperview()
        }
        for item in page.items {
            let view: NSView
            switch item.kind {
            case "heading":
                let label = NSTextField(wrappingLabelWithString: item.text)
                label.preferredMaxLayoutWidth = width - 48
                let size: CGFloat = item.level == 1 ? 28 : 20
                label.font = NSFont.boldSystemFont(ofSize: size)
                label.setAccessibilityRole(.staticText)
                label.setAccessibilityLabel(item.text)
                view = label
            case "para":
                let label = NSTextField(wrappingLabelWithString: item.text)
                label.preferredMaxLayoutWidth = width - 48
                label.setAccessibilityRole(.staticText)
                label.setAccessibilityLabel(item.text)
                view = label
            case "link":
                let button = NSButton(title: "", target: self, action: #selector(followLink(_:)))
                button.isBordered = false
                button.attributedTitle = NSAttributedString(string: item.text, attributes: [
                    .foregroundColor: NSColor.linkColor,
                    .underlineStyle: NSUnderlineStyle.single.rawValue,
                    .font: NSFont.systemFont(ofSize: NSFont.systemFontSize),
                ])
                button.identifier = NSUserInterfaceItemIdentifier(item.href)
                button.setAccessibilityRole(.button)
                button.setAccessibilityLabel(item.text)
                view = button
            default:
                continue
            }
            stack.addArrangedSubview(view)
        }
        window.setContentSize(NSSize(width: width, height: max(200, stack.fittingSize.height)))
    }

    @objc func followLink(_ sender: NSButton) {
        let href = sender.identifier?.rawValue ?? ""
        if let target = pages.first(where: { $0.route == href }) {
            show(target)
        }
    }

    func accessibilityLines() -> [String] {
        var lines: [String] = []
        for v in stack.arrangedSubviews {
            let role = v.accessibilityRole()?.rawValue ?? "none"
            let label = v.accessibilityLabel() ?? ""
            let focus = (v is NSButton) ? "focusable" : "not-focusable"
            lines.append("\(role)\t\(label)\t\(focus)")
        }
        return lines
    }

    func snapshot(to path: String) {
        window.contentView?.layoutSubtreeIfNeeded()
        guard let view = window.contentView,
              let rep = view.bitmapImageRepForCachingDisplay(in: view.bounds) else { exit(1) }
        view.cacheDisplay(in: view.bounds, to: rep)
        guard let png = rep.representation(using: .png, properties: [:]) else { exit(1) }
        do {
            try png.write(to: URL(fileURLWithPath: path))
        } catch {
            fputs("aqhost: cannot write \(path): \(error)\n", stderr)
            exit(2)
        }
    }
}

let argv = CommandLine.arguments
guard argv.count >= 2 else {
    fputs("usage: aqhost <view.json> [--route /path] [--snapshot out.png]\n", stderr)
    exit(2)
}
var route: String? = nil
var snapshotPath: String? = nil
var axDump = false
var width: CGFloat = 640
var i = 2
while i < argv.count {
    switch argv[i] {
    case "--route" where i + 1 < argv.count:
        route = argv[i + 1]; i += 2
    case "--snapshot" where i + 1 < argv.count:
        snapshotPath = argv[i + 1]; i += 2
    case "--width" where i + 1 < argv.count:
        guard let w = Double(argv[i + 1]), w >= 200 else {
            fputs("aqhost: --width needs a number of at least 200\n", stderr)
            exit(2)
        }
        width = CGFloat(w); i += 2
    case "--ax-dump":
        axDump = true; i += 1
    default:
        fputs("aqhost: unknown argument \(argv[i])\n", stderr)
        exit(2)
    }
}
let pages = loadPages(argv[1])
guard let start = pages.first(where: { route == nil ? true : $0.route == route! }) else {
    fputs("aqhost: no page for the requested route\n", stderr)
    exit(1)
}
let app = NSApplication.shared
app.setActivationPolicy(snapshotPath == nil ? .regular : .prohibited)
let host = Host(pages: pages, start: start, width: width)
if axDump {
    // Prints one line per control: role, label, and whether it can take keyboard focus.
    host.window.contentView?.layoutSubtreeIfNeeded()
    for line in host.accessibilityLines() {
        print(line)
    }
    exit(0)
}
host.window.makeKeyAndOrderFront(nil)
if let out = snapshotPath {
    host.snapshot(to: out)
    exit(0)
}
app.activate(ignoringOtherApps: true)
app.run()
