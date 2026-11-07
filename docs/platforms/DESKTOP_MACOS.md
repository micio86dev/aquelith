# Desktop target: macOS arm64

Status: **started. D1 done (route switch and link click, checked); D2 partly done (accessibility and width checked); D3 and later not started.** This is the first desktop target, chosen by the assistant under the owner's instruction to decide. Roadmap: [ROADMAP.md](ROADMAP.md).

## Design

The language does not know about windows. `aqelith view <file.aq>` checks a module and exports its pages as JSON (`compiler/src/view.rs`). A platform host draws that JSON. The macOS host is `desktop/macos/aqhost.swift`, written in Swift against AppKit, because AppKit is the platform's own interface and Swift is its own language; the host is platform code behind the target interface ([PLATFORM_ABSTRACTION.md](PLATFORM_ABSTRACTION.md)), not part of the language.

## Milestones

| Milestone | What | State | Evidence |
|---|---|---|---|
| D0 | A checked module's first page is drawn in one window: title, headings, paragraphs, links | Done | `desktop/macos/check.sh` renders routes `/`, `/how`, `/start` to PNG; the PNGs were inspected |
| D1 | A link switches to the page with that route; an unknown route fails with exit 1 | Done | `check.sh`: `aqhost --click "How it works"` opens `/how` (activation through the button's target-action, the handler a pointer click calls); `--click "No such link"` exits 1; `--route /nope` exits 1 |
| D2 | Window behaviour: text wraps when the window is resized; every control has a role and a label; every link is a focusable button | Partly done | `check.sh`: accessibility per route (links, labels), width (a narrow window is taller). Keyboard focus order and VoiceOver are not checked |
| D3 | Renderer conformance (C3, [CONFORMANCE.md](CONFORMANCE.md) section 4) | Not started | — |
| D4 | Signed application bundle, install and uninstall | Not started | — |

## Limits, as they are today

- The host draws a static view: no state, no events beyond link navigation, no text input.
- The click check does not send an OS pointer event. It calls the button's target-action, which is the same handler a mouse click runs. A real mouse click through the window server, and a click on the rendered coordinates, are not verified.
- Only macOS arm64 on this machine has been run. Intel macOS has not.
- The JSON export is not yet a stable contract for other hosts; it is versioned by its use here.
- No performance claim is made ([../performance/BASELINE.md](../performance/BASELINE.md)).

## How to check

```text
bash desktop/macos/check.sh
```
