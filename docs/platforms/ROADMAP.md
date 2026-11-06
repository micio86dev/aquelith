# Platform roadmap (every target, in order)

Status: **Proposed.** Each target below starts only when its feature document is written and approved. Nothing here is implemented beyond what each row says. Support criteria: [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 3. Conformance levels: [CONFORMANCE.md](CONFORMANCE.md).

## Status today

| Target | State | Evidence |
|---|---|---|
| Web (browser), static pages | Built | `web/` tests, landing browser test |
| Server, loopback HTTP, macOS arm64 | Minimal. C2 (loopback HTTP) passes; C1 passes on 4 corpus files, expected outputs pending owner review | `compiler/tests/serve.rs`, `conformance_server.rs`, [SERVER_CONFORMANCE.md](SERVER_CONFORMANCE.md) |
| Server, Linux x86_64 | Minimal. Conformance passed 5 of 5 under emulation (`linux/amd64` container); native run pending | [SERVER_CONFORMANCE.md](SERVER_CONFORMANCE.md) |
| Desktop, macOS arm64 | Started: static view and route switch done; click path unverified; D2 partly done. See [DESKTOP_MACOS.md](DESKTOP_MACOS.md) | `desktop/macos/check.sh` |
| Desktop, Linux | Started: GTK host draws the pages in a container; not a native desktop yet. See [DESKTOP_LINUX.md](DESKTOP_LINUX.md) | `desktop/linux/check.sh` |
| Desktop, Windows | No backend | none |
| Mobile, iOS | Proposed. Blocked: full Xcode not installed | [MOBILE_IOS.md](MOBILE_IOS.md) |
| Mobile (iOS, Android) | No backend | none |
| Embedded and constrained | No backend | none |
| Other operating systems | No backend | none |

## Order and dependencies

1. **Linux x86_64 server, conformance run.** Done under emulation: `conformance_server` passed 5 of 5 in a `linux/amd64` Docker container. A native Linux x86_64 machine is still needed before a Linux support claim.
2. **Server production hardening.** The host is a loopback development host. Needed before any server target is called supported: TLS or an explicit non-loopback decision, authentication, and a process model that is not one connection at a time (threat model, residual risks).
3. **Record types in the language beyond request and response.** Needed by every target that exchanges data. Records are implemented for v0 ([RECORDS.md](../language/RECORDS.md)).
4. **Desktop (one OS first).** Requires a native window interface, an event loop, and a renderer conformance suite (C3). Choose one operating system first; the other two follow the same interface.
5. **Mobile (one OS first).** Requires a native UI, not a WebView wrapper ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4).
6. **Embedded.** Requires the no-heap profile ([MEMORY_MODEL.md](../language/MEMORY_MODEL.md)) and a feasibility study on a real board. The reference audit found Rust `no_std` only ([REFERENCE_ECOSYSTEMS.md](../research/REFERENCE_ECOSYSTEMS.md), and [TARGET_MATRIX.md](TARGET_MATRIX.md) row 'Embedded RTOS').
7. **Other operating systems.** Each one is its own feature document, after the target it resembles.

## Decisions the owner must make

- Installing full Xcode for the iOS target ([MOBILE_IOS.md](MOBILE_IOS.md)). The first mobile operating system is iOS, chosen by the assistant; Android is later. The first desktop operating system is macOS arm64 ([DESKTOP_MACOS.md](DESKTOP_MACOS.md)).
- The first mobile operating system (iOS or Android).
- Whether to provide a Linux x86_64 runner, and where.
- The memory model for the first non-web target (decision D3 in SERVER_LINUX_PLAN.md).

## What "any operating system and device" means here

It is the goal, not a finished state. Each target is a separate project with its own milestones. This roadmap does not claim that any of them works.
