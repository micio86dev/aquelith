# Platform roadmap (every target, in order)

Status: **Proposed.** Each target below starts only when its feature document is written and approved. Nothing here is implemented beyond what each row says. Support criteria: [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 3. Conformance levels: [CONFORMANCE.md](CONFORMANCE.md).

## Status today

| Target | State | Evidence |
|---|---|---|
| Web (browser), static pages | Built | `web/` tests, landing browser test |
| Server, loopback HTTP, macOS arm64 | Minimal. C2 (loopback HTTP) passes; C1 passes on 4 corpus files, expected outputs pending owner review | `compiler/tests/serve.rs`, `conformance_server.rs`, [SERVER_CONFORMANCE.md](SERVER_CONFORMANCE.md) |
| Server, Linux x86_64 | Minimal. The compiler suite (`cargo test -q`) passed natively on x86_64 Linux in GitHub Actions run 42358884309 (`ubuntu-latest`). Earlier 5 of 5 was under emulation and is not counted as native | [SERVER_CONFORMANCE.md](SERVER_CONFORMANCE.md), `.github/workflows/linux-x86_64.yml` |
| Desktop, macOS arm64 | Started: static view, route switch and link click (target-action) checked; a real OS pointer click not verified; D2 partly done. See [DESKTOP_MACOS.md](DESKTOP_MACOS.md) | `desktop/macos/check.sh` |
| Desktop, Linux | Started: GTK host draws the pages and navigates by link, in a container; a real pointer click and a native Linux desktop are not verified. See [DESKTOP_LINUX.md](DESKTOP_LINUX.md) | `desktop/linux/check.sh` |
| Desktop, Windows | Compiler suite passes natively on Windows x64 (GitHub Actions run 42365115292). The desktop host is not started. No local Windows machine needed. Wine is not used | [DESKTOP_WINDOWS.md](DESKTOP_WINDOWS.md) |
| Mobile, Android | Proposed, first mobile target (owner decision, 2026-11-07). No backend yet. Blocked: the SDK download needs the owner's OK and disk space is low | [MOBILE_ANDROID.md](MOBILE_ANDROID.md) |
| Mobile, iOS | Proposed, second mobile target (after Android). Blocked: full Xcode not installed (disk space) | [MOBILE_IOS.md](MOBILE_IOS.md) |
| Embedded and constrained | No backend | none |
| Other operating systems | No backend | none |

## Order and dependencies

1. **Linux x86_64 server, conformance run.** Done natively: GitHub Actions run 42358884309 (workflow `linux-x86_64`, `ubuntu-latest`, host `Linux 7.0.0-1016-aws x86_64`) passed `cargo test -q` for the compiler. The earlier 5 of 5 in a `linux/amd64` Docker container was emulation and is not counted as native.
2. **Server production hardening.** Partly done for the loopback target. Done: the process model is concurrent (one thread per connection, cap of 32, tests `a_slow_client_does_not_block_another_client` and `connections_past_the_cap_get_503_at_once`). Decided: the server stays loopback-only, so TLS and authentication are not added; a non-loopback target needs its own threat model ([SERVE.md](../architecture/SERVE.md), [SERVER_THREAT_MODEL.md](../security/SERVER_THREAT_MODEL.md)). Open: an automated test for T8 (a panic in one request) and no third-party audit.
3. **Record types in the language beyond request and response.** Needed by every target that exchanges data. Records are implemented for v0 ([RECORDS.md](../language/RECORDS.md)).
4. **Desktop (one OS first).** Requires a native window interface, an event loop, and a renderer conformance suite (C3). Choose one operating system first; the other two follow the same interface. The first desktop operating system is macOS arm64 ([DESKTOP_MACOS.md](DESKTOP_MACOS.md)); Linux GTK is started in parallel.
5. **Mobile (one OS first).** Requires a native UI, not a WebView wrapper ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4).
6. **Embedded.** Requires the no-heap profile ([MEMORY_MODEL.md](../language/MEMORY_MODEL.md)) and a feasibility study on a real board. The reference audit found Rust `no_std` only ([REFERENCE_ECOSYSTEMS.md](../research/REFERENCE_ECOSYSTEMS.md), and [TARGET_MATRIX.md](TARGET_MATRIX.md) row 'Embedded RTOS').
7. **Other operating systems.** Each one is its own feature document, after the target it resembles.

## Decisions the owner must make

- Installing full Xcode for the iOS target ([MOBILE_IOS.md](MOBILE_IOS.md)). The owner has deferred it: disk space is too low for it now.
- The first mobile target is Android, then iOS (owner decision, 2026-11-07). Starting Android needs its feature document approved and the SDK download approved by the owner.
- The memory model for the first non-web target (decision D3 in SERVER_LINUX_PLAN.md).

## What "any operating system and device" means here

It is the goal, not a finished state. Each target is a separate project with its own milestones. This roadmap does not claim that any of them works.
