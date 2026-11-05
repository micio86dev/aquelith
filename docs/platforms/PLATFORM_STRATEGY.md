# AQELITH Platform Strategy

Status: **Proposed (M2).** No platform is supported yet. A platform becomes supported only when all five conditions in §3 hold (directive §8.4).

## 1. Principles

- One language and one semantic core; platform-specific code lives behind target interfaces ([PLATFORM_ABSTRACTION.md](PLATFORM_ABSTRACTION.md)).
- Share application logic and semantics across targets; use specialized renderers and runtimes where needed. One runtime everywhere is not a goal.
- Platform features are reached through declared capabilities, never ambient access.
- The web backend does not contaminate the language core (directive §16).

## 2. Staged order

| Stage | Target | Why this order |
|---|---|---|
| 1 | Web (browser) | Directive §16: the first complete proof of concept is Web. |
| 2 | Server (Linux x86_64, ARM64) | Shares the interpreter and IR; needed for full-stack apps. |
| 3 | Desktop (macOS, Linux, Windows) | Native windowing through target interfaces; no WebView-only strategy. |
| 4 | Mobile (iOS, Android) | Native UI where appropriate; Naze's Android WebView wrapper is the rejected baseline. |
| 5 | Embedded and constrained | Requires the no-heap profile from [../language/MEMORY_MODEL.md](../language/MEMORY_MODEL.md). |

Stages 3 to 5 are not scheduled. They are listed so that the core is designed not to block them.

## 3. Support criteria (all required)

1. The backend for the target exists in the repository.
2. Runtime requirements are documented.
3. Conformance tests pass on the target ([CONFORMANCE.md](CONFORMANCE.md)).
4. Security assumptions are documented ([../security/SECURITY_ARCHITECTURE.md](../security/SECURITY_ARCHITECTURE.md)).
5. Limitations are listed in the target matrix ([TARGET_MATRIX.md](TARGET_MATRIX.md)).

## 4. What is explicitly rejected

- Canvas2D as the only web renderer (directive §8.3; the Naze audit found the canvas-only path bypasses the semantic DOM).
- WebView as the only mobile or desktop strategy.
- Claiming "universal" support for targets that have no backend.

## 5. Open questions

- Whether the server target runs the same interpreter as development or a separately compiled artifact.
- Choice of native windowing libraries; deferred until M7-era evaluation.
