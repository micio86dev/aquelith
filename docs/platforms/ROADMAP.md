# Platform roadmap (every target, in order)

Status: **Proposed.** Each target below starts only when its feature document is written and approved. Nothing here is implemented beyond what each row says. Support criteria: [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 3. Conformance levels: [CONFORMANCE.md](CONFORMANCE.md).

## Status today

| Target | State | Evidence |
|---|---|---|
| Web (browser), static pages | Built | `web/` tests, landing browser test |
| Server, loopback HTTP, macOS arm64 | Minimal. C2 (loopback HTTP) passes; C1 passes on 4 corpus files, expected outputs pending owner review | `compiler/tests/serve.rs`, `conformance_server.rs`, [SERVER_CONFORMANCE.md](SERVER_CONFORMANCE.md) |
| Server, Linux x86_64 | Minimal, not run | Needs a runner |
| Desktop (macOS, Linux, Windows) | No backend | none |
| Mobile (iOS, Android) | No backend | none |
| Embedded and constrained | No backend | none |
| Other operating systems | No backend | none |

## Order and dependencies

1. **Linux x86_64 server, conformance run.** Blocked on a runner, not on code. Milestone: run `conformance_server` on a Linux x86_64 machine and record the output in SERVER_CONFORMANCE.md.
2. **Server production hardening.** The host is a loopback development host. Needed before any server target is called supported: TLS or an explicit non-loopback decision, authentication, and a process model that is not one connection at a time (threat model, residual risks).
3. **Record types in the language beyond request and response.** Needed by every target that exchanges data. Records are implemented for v0 ([RECORDS.md](../language/RECORDS.md)).
4. **Desktop (one OS first).** Requires a native window interface, an event loop, and a renderer conformance suite (C3). Choose one operating system first; the other two follow the same interface.
5. **Mobile (one OS first).** Requires a native UI, not a WebView wrapper ([PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 4).
6. **Embedded.** Requires the no-heap profile ([MEMORY_MODEL.md](../language/MEMORY_MODEL.md)) and a feasibility study on a real board. The reference audit found Rust `no_std` only ([REFERENCE_ECOSYSTEMS.md](../research/REFERENCE_ECOSYSTEMS.md), and [TARGET_MATRIX.md](TARGET_MATRIX.md) row 'Embedded RTOS').
7. **Other operating systems.** Each one is its own feature document, after the target it resembles.

## Decisions the owner must make

- The first desktop operating system (macOS, Linux or Windows).
- The first mobile operating system (iOS or Android).
- Whether to provide a Linux x86_64 runner, and where.
- The memory model for the first non-web target (decision D3 in SERVER_LINUX_PLAN.md).

## What "any operating system and device" means here

It is the goal, not a finished state. Each target is a separate project with its own milestones. This roadmap does not claim that any of them works.
