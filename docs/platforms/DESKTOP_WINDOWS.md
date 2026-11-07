# Desktop target: Windows (x86_64)

Status: **proposed. Blocked: no Windows machine, and no free way to run Windows here.** Decided by the assistant as the next desktop target after Linux, under the owner's instruction to continue with Linux and then Windows.

## Blockers, checked 2026-11-07

- This Mac is arm64. Windows does not run on it natively, and a Windows virtual machine needs a paid licence or a Windows installer the owner must provide.
- Wine, the usual way to run Windows programs on Linux, is in Debian for x86_64 only. On this arm64 Mac it would run under x86_64 emulation, which is slow and large. The internal disk had 4.4 GiB free at this check.
- A Windows container needs a Windows host. Docker cannot provide one here.

## Options (the owner chooses one)

1. **A Windows x86_64 machine** (or a VM on one): native run, the strongest evidence. Needs the owner's machine or a remote one.
2. **Wine under x86_64 emulation in Docker**: free, but slow, large (several GB), and not a Windows run. It would be labelled as Wine, like the Linux x86_64 emulation.

## Design, once a machine exists

The same split as the other desktop targets ([DESKTOP_MACOS.md](DESKTOP_MACOS.md), [DESKTOP_LINUX.md](DESKTOP_LINUX.md)): `aqelith view` exports the pages as JSON, and a Win32 host draws them. A native Win32 interface is required; a WebView-only wrapper is the rejected baseline ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4). The JSON export needs no change.

## Milestones

| Milestone | What | Acceptance | State |
|---|---|---|---|
| W0 | A Windows x86_64 environment (option 1 or 2) | `cmd /c ver` or equivalent prints the Windows version | Blocked (owner choice) |
| W1 | A Win32 host draws a checked module's first page | A screenshot from that environment, inspected | Not started |
| W2 | Link navigation between pages | A click changes the page; the route is checked | Not started |
| W3 | Accessibility: every control has a name and a role; keyboard focus reaches every link | An accessibility check run on Windows | Not started |
| W4 | A signed installer | Signing needs a certificate, which may be paid: the owner decides | Not started |

## Limits

- Nothing is built or run on Windows yet. No claim is made.
- A Wine run, if chosen, is not evidence for Windows.
