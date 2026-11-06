# Desktop target: macOS arm64

Status: **started. Milestone D1 done; D2 and later not started.** This is the first desktop target, chosen by the assistant under the owner's instruction to decide. Roadmap: [ROADMAP.md](ROADMAP.md).

## Design

The language does not know about windows. `aqelith view <file.aq>` checks a module and exports its pages as JSON (`compiler/src/view.rs`). A platform host draws that JSON. The macOS host is `desktop/macos/aqhost.swift`, written in Swift against AppKit, because AppKit is the platform's own interface and Swift is its own language; the host is platform code behind the target interface ([PLATFORM_ABSTRACTION.md](PLATFORM_ABSTRACTION.md)), not part of the language.

## Milestones

| Milestone | What | State | Evidence |
|---|---|---|---|
| D0 | A checked module's first page is drawn in one window: title, headings, paragraphs, links | Done | `desktop/macos/check.sh` renders routes `/`, `/how`, `/start` to PNG; the PNGs were inspected |
| D1 | A link switches to the page with that route; an unknown route fails with exit 1 | Done for the route option and the switch in code | `aqhost --route /how` and `--route /nope`; the click path itself is not automated |
| D2 | Real window behaviour: resizing, keyboard navigation to every link, focus, accessibility names for every control | Not started | — |
| D3 | Renderer conformance (C3, [CONFORMANCE.md](CONFORMANCE.md) section 4) | Not started | — |
| D4 | Signed application bundle, install and uninstall | Not started | — |

## Limits, as they are today

- The host draws a static view: no state, no events beyond link navigation, no text input.
- A link click is implemented but has not been driven by an automated test; only the route switch is checked.
- Only macOS arm64 on this machine has been run. Intel macOS has not.
- The JSON export is not yet a stable contract for other hosts; it is versioned by its use here.
- No performance claim is made ([../performance/BASELINE.md](../performance/BASELINE.md)).

## How to check

```text
bash desktop/macos/check.sh
```
