# Project Status

Last updated: 2026-11-06

| Milestone | State |
|---|---|
| M0 Environment and repository | Done (commit c8abf1b) |
| M1 Competitive research | Complete with documented gaps (see docs/reports/M1_RESEARCH_REPORT.md): Astra source private, Zerolang test suite incomplete, three AICore test failures unexplained |
| M2 Architecture synthesis | Complete as proposals: language, security, compiler, platform, performance, testing and application specs; 8 ADRs; risk register; 23 open questions. See docs/reports/M2_ARCHITECTURE_REPORT.md |
| M3 Feasibility prototypes | Complete with 2 prototypes built (stable identities and patches; capability values) and 6 candidates deferred with reasons. See docs/reports/M3_FEASIBILITY_REPORT.md |
| M4 Language and compiler foundation | Done for the v0 subset (lexer, parser, checker, formatter, CLI, 15 tests). Full grammar still open. See docs/reports/M4_M8_REPORT.md |
| M5 Semantic graph and agent interface | Partial: query, JSON diagnostics, and a patch slice (`aqelith patch`, v1: rename, set-return, add-effect, remove-effect; base hash, preconditions, full-checker validation, atomic write, dry run; 16 tests). Not built: replace/insert/delete, persistent ids in the compiler, deterministic repair |
| M6 Web application runtime | Partial: first vertical slice (counter) with semantic renderer, declarative events, per-cell repaint, 10 render tests, 3 runtime tests that execute the browser script against a DOM stub (`node --test web/tests/`), manual Chrome check. Forms, routing, data, SSR of app state, accessibility checks and browser tests in CI not built |
| M7 Testing, security, performance | Partial: corpus, property, mutation and deterministic fuzz tests (`compiler/tests/fuzz.rs`: fixed-seed mutation of sources, JSON and patches, checking no panic, formatter idempotence, and that accepted patches pass the checker). Not built: coverage-guided fuzzing in CI, sanitizers (no runtime yet), benchmarks |
| M8 Agent ecosystem | Partial: skill for the v0 CLI with a drift test. No MCP server |
| M9 Public developer preview | Blocked: brand approval (gate) |

## Blocked
- Public branding: pending user approval (brand gate).

## Rejected / Deferred
- Deferred: parser comparison, IR benchmarks, incremental compilation, web rendering prototype, serialization prototype, runtime capability enforcement (reasons in M3 report).
- Rejected as design: see docs/strategy/REJECTED_IDEAS.md.
