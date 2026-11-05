# Project Status

Last updated: 2026-11-05

| Milestone | State |
|---|---|
| M0 Environment and repository | Done (commit c8abf1b) |
| M1 Competitive research | Complete with documented gaps (see docs/reports/M1_RESEARCH_REPORT.md): Astra source private, Zerolang test suite incomplete, three AICore test failures unexplained |
| M2 Architecture synthesis | Complete as proposals: language, security, compiler, platform, performance, testing and application specs; 8 ADRs; risk register; 23 open questions. See docs/reports/M2_ARCHITECTURE_REPORT.md |
| M3 Feasibility prototypes | Complete with 2 prototypes built (stable identities and patches; capability values) and 6 candidates deferred with reasons. See docs/reports/M3_FEASIBILITY_REPORT.md |
| M4 Language and compiler foundation | Done for the v0 subset (lexer, parser, checker, formatter, CLI, 15 tests). Full grammar still open. See docs/reports/M4_M8_REPORT.md |
| M5 Semantic graph and agent interface | Partial: query and JSON diagnostics. Patch API over the IR not built |
| M6 Web application runtime | Not done: needs a renderer design decision first |
| M7 Testing, security, performance | Partial: corpus, property and mutation tests. No fuzzing in CI, no sanitizers, no benchmarks |
| M8 Agent ecosystem | Partial: skill for the v0 CLI with a drift test. No MCP server |
| M9 Public developer preview | Blocked: brand approval (gate) |

## Blocked
- Public branding: pending user approval (brand gate).

## Rejected / Deferred
- Deferred: parser comparison, IR benchmarks, incremental compilation, web rendering prototype, serialization prototype, runtime capability enforcement (reasons in M3 report).
- Rejected as design: see docs/strategy/REJECTED_IDEAS.md.
