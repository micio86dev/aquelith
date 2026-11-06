# Project Status

Last updated: 2026-11-06

| Milestone | State |
|---|---|
| M0 Environment and repository | Done (commit c8abf1b) |
| M1 Competitive research | Complete with documented gaps (see docs/reports/M1_RESEARCH_REPORT.md): Astra source private, Zerolang test suite incomplete, three AICore test failures unexplained |
| M2 Architecture synthesis | Complete as proposals: language, security, compiler, platform, performance, testing and application specs; 8 ADRs; risk register; 23 open questions. See docs/reports/M2_ARCHITECTURE_REPORT.md |
| M3 Feasibility prototypes | Complete with 2 prototypes built (stable identities and patches; capability values) and 6 candidates deferred with reasons. See docs/reports/M3_FEASIBILITY_REPORT.md |
| M4 Language and compiler foundation | Done for the v0 subset (lexer, parser, checker, formatter, CLI, 15 tests). Full grammar still open. See docs/reports/M4_M8_REPORT.md |
| M5 Semantic graph and agent interface | Partial: query, JSON diagnostics, and a patch slice (`aqelith patch`, v1: rename, set-return, add-effect, remove-effect, and function-level delete, replace and insert with content-hash preconditions; base hash, full-checker validation, atomic write, dry run; 25 tests). Persistent ids for functions (`aqelith ids`, sidecar index, `id:` targets, 9 tests: 3 unit and 6 integration; rename keeps the id, ids not reused in practice). Also `set-param-type`, and `aqelith repair`, a deterministic proposal of the effects calls need (fixed point, verified by the patch API; 5 repair tests). Not built: expression-level edits, node-level ids, repair of type errors |
| M6 Web application runtime | Partial: static multi-page site with routing and same-site links (ADR-0014; `web/tests/site.rs`, 6 tests), counter and integer input (ADR-0012, ADR-0013), accessibility checks at build (title required, heading findings; `web/tests/render.rs`), runtime and browser tests in headless Chrome (`node --test web/tests/runtime.test.mjs web/tests/browser/run.mjs`; the browser test skips without Chrome). Not built: client-side navigation, SSR and streaming, resumability, keyboard and screen-reader testing, CI execution |
| M7 Testing, security, performance | Partial: AddressSanitizer run of all compiler tests with a nightly toolchain (no reports; the binaries carry ASan symbols); a benchmark skeleton that prints raw front-end timings and claims no numbers (`compiler/examples/bench_frontend.rs`); a CI workflow written but not run (no remote; `.github/workflows/ci.yml`); corpus, property, mutation and deterministic fuzz tests (`compiler/tests/fuzz.rs`: fixed-seed mutation of sources, JSON and patches, checking no panic, formatter idempotence, and that accepted patches pass the checker). Coverage-guided fuzzing built as two libFuzzer targets (`compiler/fuzz`): 60-second runs on nightly, macOS arm64, 8.9M executions for the front end and 6.6M for patches, no crash. Not built: long campaigns and a persistent corpus in CI, CI runs, reference-machine benchmarks (Q22), a sanitized runtime |
| M8 Agent ecosystem | Partial: skill for the v0 CLI with a drift test; `aqelith-mcp`, an MCP server over stdio (protocol 2024-11-05) with check, format, query, explain and patch tools that take text and never touch files (11 tests). HTTP transport on loopback (`aqelith-mcp --http`, Host-header guard; 5 tests). Not built: versioned machine-readable docs beyond the skill, agent productivity benchmarks, non-loopback serving |
| M9 Public developer preview | Blocked: brand approval (gate) |

## Blocked
- Public branding: pending user approval (brand gate).

## Rejected / Deferred
- Deferred: parser comparison, IR benchmarks, incremental compilation, web rendering prototype, serialization prototype, runtime capability enforcement (reasons in M3 report).
- Rejected as design: see docs/strategy/REJECTED_IDEAS.md.
