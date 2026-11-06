# Mobile target: iOS (simulator first)

Status: **proposed. Blocked on a toolchain that is not installed.** This is the first mobile target, chosen by the assistant under the owner's instruction to decide.

## Blocker, found 2026-11-06

This machine has the Command Line Tools only. `xcodebuild` reports that it "requires Xcode", `xcrun simctl` is missing, and there are no iOS simulator runtimes. An earlier note said Xcode was installed; that was wrong. The owner must install the full Xcode application (a large download from the App Store or developer.apple.com), and then run `sudo xcode-select -s /Applications/Xcode.app`. Only then can the milestones below start.

## Design, once the toolchain exists

The same split as the desktop target ([DESKTOP_MACOS.md](DESKTOP_MACOS.md)): `aqelith view` exports the pages as JSON, and a Swift host draws them with UIKit. The language does not know about iOS. A native UIKit view is required; a WebView-only wrapper is the rejected baseline ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4).

## Milestones

| Milestone | What | Acceptance | State |
|---|---|---|---|
| I0 | Full Xcode installed and selected; an iOS simulator runtime present | `xcodebuild -version` prints Xcode; `xcrun simctl list runtimes` lists iOS | Blocked (owner action) |
| I1 | A UIKit host draws a checked module's first page on the simulator | A screenshot from the simulator, inspected; the check script fails when the page is missing | Not started |
| I2 | Link navigation between pages | A tap changes the page; the route is checked | Not started |
| I3 | Accessibility labels for every control; Dynamic Type | An accessibility audit in the simulator reports no unlabelled control | Not started |
| I4 | A physical iPhone run and signing | A build installed on a device the owner holds | Not started; needs an Apple developer account |

## Limits

- Nothing is built or run on iOS yet. No claim is made.
- The simulator is not a device: performance and battery are not measured there.
- Signing and distribution need an Apple developer account, which is a paid service: the owner decides before it is used ([AGENTS.md](../../AGENTS.md) gates).
