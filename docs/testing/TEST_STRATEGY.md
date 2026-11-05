# AQELITH Test Strategy

Status: **Proposed (M2).** No tests exist yet; this document defines the layers and the rules every feature must meet. Directive §14.

## 1. Layers

| Layer | What it covers | Tooling (proposed) |
|---|---|---|
| Lexer | Tokens, spans, error recovery | Rust unit tests |
| Parser | Accept and reject corpora, round-trip | Rust tests, golden files |
| Type checker | Each error code has a minimal failing input | Rust tests, `compile_fail`-style fixtures |
| IR | Determinism: same source, same bytes | Golden files, property tests |
| Semantic graph | Id stability under rename and format | Property tests |
| Patch validation | Stale rejection, precondition rejection, atomicity | Rust tests (see prototype P1) |
| Compiler end to end | Build and run corpus | Golden outputs |
| Runtime | Scheduling, cancellation, cleanup order | Deterministic harness |
| Backend conformance | Same corpus, same output | [CONFORMANCE_STRATEGY.md](CONFORMANCE_STRATEGY.md) |
| Unit and integration (app) | Components headless | Rust or AQELITH test runner |
| Browser E2E | Generated web apps | Playwright (directive §14) |
| Accessibility | Semantic and keyboard checks | axe-style rules plus manual checklist |
| Security | See [SECURITY_TESTING.md](SECURITY_TESTING.md) | |
| Fuzzing | Parser, IR deserializer, patch parser, runtime codec | cargo-fuzz or equivalent (open choice) |
| Property-based | Round-trip, idempotence, id stability | property test library (open choice) |
| Performance | See [BENCHMARK_TESTING.md](BENCHMARK_TESTING.md) | |

## 2. Rules

1. A feature is not complete without tests and documentation (directive §0.1).
2. Every error code has at least one test that triggers it.
3. A test asserts every observable effect the rule names: output, exit code, persisted data.
4. Tests go through the public interface, not private internals.
5. A skipped test is reported as skipped, never as passed.
6. Critical invariants have explicit tests even when coverage is high.

## 3. Coverage

Target of at least 85% for meaningful compiler and application logic (directive §14), measured by branch coverage, not line count. The directive forbids optimizing for meaningless coverage; coverage is a signal, not a goal.

## 4. Gates

Tests run in CI before merge (directive §17.1). Formatting, linting and static analysis gates apply to every language.

## 5. Where tests come from

Each competitor failure reproduced in the audits becomes a regression test of the same class in AQELITH (for example the Zerolang deep-input crash, the Naze hydration escape, the Edict missing-effect across imports).
