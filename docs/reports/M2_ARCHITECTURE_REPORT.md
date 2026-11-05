# M2 Architecture Synthesis Report

Date: 2026-11-05. Status: **M2 documents written and reviewed for links; decisions are Proposed unless marked.** No code of the language exists yet; M4 starts only after the open questions listed below are reviewed.

## 1. What was completed

- Language specifications (directive §4): overview, grammar draft, type system, error model, concurrency model, memory model.
- Security specifications (§6): threat model with 18 threats and their mitigation status, security architecture, guarantees (split into compile-time, runtime, OS, not-guaranteed), capability model, effect system.
- Performance specifications (§7): model, budgets (deliberately unset), methodology.
- Platform specifications (§8): strategy, target matrix (all planned), platform abstraction, conformance levels.
- Compiler specifications (§10): canonical IR, semantic graph, semantic patch API, stable identities.
- Application model and resumability (§9).
- Testing specifications (§14) and benchmark README (§15).
- Architecture overview with diagrams, eight ADRs, risk register (17 risks), open questions (23).

## 2. Important architecture decisions

| ADR | Decision | Status |
|---|---|---|
| 0001 | Rust core; TypeScript only for browser glue | Proposed |
| 0002 | Text is the authority; IR derived; patches reprojected through the formatter and re-checked | Proposed |
| 0003 | Capabilities are values, attenuable, never serialized | Proposed, provisional (P2) |
| 0004 | Persistent opaque ids plus content hashes | Proposed, provisional (P1) |
| 0005 | Interpreter first; web JS; WASM for compute; no LLVM in v1 | Proposed |
| 0006 | Web renders the semantic DOM | Accepted as constraint; mechanics proposed |
| 0007 | Resumability limited to declared state and handler ids | Proposed |
| 0008 | Single owner, lexical borrows, no GC, no lifetimes in v1 | Proposed |

## 3. Files created

`docs/language/` (6), `docs/security/` (5), `docs/performance/` (3), `docs/platforms/` (4), `docs/compiler/` (4), `docs/app/` (2), `docs/testing/` (4), `docs/adr/` (8), `docs/architecture/` (3), `benchmarks/README.md`, `docs/reports/M2_ARCHITECTURE_REPORT.md`.

## 4. Tests executed

M2 is documents. The only executable checks are the prototype tests in M3 ([M3_FEASIBILITY_REPORT.md](M3_FEASIBILITY_REPORT.md)). A relative-link check over `docs/` found 4 broken links, all pointing at prototype README files that were created afterwards; after that they resolve.

## 5. Gates and reviews

GGA review per commit, as in M1. Results are recorded in the commit log and in the task file, not repeated here.

## 6. Known limitations

- Language syntax is illustrative. Nothing has been parsed.
- Several ADRs rest on competitor audits, which were partly on static reading (see M1 report §5).
- Security guarantees are targets; no test enforces them yet.
- Budgets are not set; no reference machine is declared.

## 7. Remaining risks (top five, from [../architecture/RISK_REGISTER.md](../architecture/RISK_REGISTER.md))

- R1 brand conflict (gate open).
- R2 graph and text divergence (ADR-0002 has no formatter prototype yet).
- R4 capability ergonomics.
- R6 resumability deployment skew.
- R8 competitor evidence ages.

## 8. Open questions that block M4

Q1 coherence, Q3 lifetimes, Q6 syntax details (operators, closures, `?`), Q7 error-propagation syntax, Q10 formatter round trip, Q17 user-declared effects, Q23 root capability minting. These are the ones an M4 parser and checker cannot avoid. M4 can start with the subset that does not depend on them, but the grammar draft must be revised first.

## 9. Next milestone

M4 (language and compiler foundation). Recommendation: after M3 prototypes are reviewed and Q6 and Q10 have a proposed answer, implement the lexer and parser for the core subset with the accept and reject corpora from GRAMMAR §4.
