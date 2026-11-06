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
| M6 Web application runtime | Partial: static multi-page site with routing and same-site links (ADR-0014; `web/tests/site.rs`, 6 tests), counter and integer input (ADR-0012, ADR-0013), accessibility checks at build (title required, heading findings; `web/tests/render.rs`), runtime and browser tests in headless Chrome (`node --test web/tests/runtime.test.mjs web/tests/browser/run.mjs`; the browser test skips without Chrome). Static build minifies HTML and CSS and sets a Content-Security-Policy (`web/src/minify.rs`, `web/tests/static_build.rs`; sizes in `docs/performance/SIZE_BASELINE.md`, no budget yet). Dev server with live reload (`web/examples/dev_site.rs`, `web/src/dev.rs`, `web/runtime/reload.js`): rebuilds on save, shows errors in a banner, serves from memory on 127.0.0.1 with a Host guard (`web/tests/dev_server.rs` 17 tests, `web/tests/reload.test.mjs` 6 tests). Not built: client-side navigation, SSR and streaming, resumability, keyboard and screen-reader testing, CI execution, a production server |
| M7 Testing, security, performance | Partial: AddressSanitizer run of all compiler tests with a nightly toolchain (no reports; the binaries carry ASan symbols); a benchmark skeleton that prints raw front-end timings and claims no numbers (`compiler/examples/bench_frontend.rs`); a CI workflow written but not run (no remote; `.github/workflows/ci.yml`); corpus, property, mutation and deterministic fuzz tests (`compiler/tests/fuzz.rs`: fixed-seed mutation of sources, JSON and patches, checking no panic, formatter idempotence, and that accepted patches pass the checker). Coverage-guided fuzzing built as two libFuzzer targets (`compiler/fuzz`): 60-second runs on nightly, macOS arm64, 8.9M executions for the front end and 6.6M for patches, no crash. Not built: long campaigns and a persistent corpus in CI, CI runs, reference-machine benchmarks (Q22), a sanitized runtime |
| M8 Agent ecosystem | Partial: skill for the v0 CLI with a drift test; `aqelith-mcp`, an MCP server over stdio (protocol 2024-11-05) with check, format, query, explain and patch tools that take text and never touch files (11 tests). HTTP transport on loopback (`aqelith-mcp --http`, Host-header guard; 5 tests). Not built: versioned machine-readable docs beyond the skill, agent productivity benchmarks, non-loopback serving |
| M9 Public developer preview | Blocked: brand approval (gate) |

## Blocked
- Public branding: pending user approval (brand gate).

## Rejected / Deferred
- Deferred: parser comparison, IR benchmarks, incremental compilation, web rendering prototype, serialization prototype, runtime capability enforcement (reasons in M3 report).
- Rejected as design: see docs/strategy/REJECTED_IDEAS.md.

## Local experiment: landing page slice (ADR-0015)

Done and tested locally: string literals, `page` items with a closed statement set, neutral page checks (PAGE001 to PAGE005, LEX004, LEX005), canonical formatting of pages, function patches that keep pages, a static web backend in `web/` (`site_from_source`), an example landing site in `web/examples/landing/landing.aq`, and a headless Chrome test that checks every route and link. Styles: one stylesheet per site, linked from every page (`build_site <file.aq> <out> <style.css>`); it is a backend asset, not language syntax. Speed comparison: two fresh agent runs per arm were done (docs/reports/LANDING_SPEED_RESULTS.md); no measurable difference at this sample size. Not done: the owner's manual arm and a third run per arm, a second backend (mobile, desktop), escapes, string values in functions, images and lists.
