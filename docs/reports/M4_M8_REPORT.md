# M4 to M8 Report: what was done, and what was not

Date: 2026-11-05. Scope requested by the owner: "procedi, scegli tu, non fermarti mai finché non completi tutto". This report is honest about completion: each milestone is marked **Done**, **Partial** or **Not done**, with the reason.

## Status by milestone

| Milestone | Status | What exists | What is missing |
|---|---|---|---|
| M4 Language and compiler foundation | **Done for the v0 subset** | Lexer, parser, checker (names, types, effects, duplicates), canonical formatter, `check`/`fmt`/`query`/`explain` CLI, stable diagnostic codes, 15 tests | Full grammar (see GRAMMAR.md), comments, closures, strings, `?`, imports, generics, traits, a real IR, incremental checking, resumable parsing |
| M5 Semantic graph and agent interface | **Partial** | `query` (callers, callees, effects per function), JSON diagnostics, skill file | Patch API over the IR (SEMANTIC_PATCH_API), persistent ids in the compiler (only the toy prototype P1 has them), deterministic repair engine, graph hash in responses |
| M6 Web application runtime | **Not done** | Nothing | Components, reactivity, DOM renderer, routing, SSR, resumability. Needs a renderer design (ADR-0006 is a constraint, not a design), a browser harness, and a runtime for the application model. Starting it now would produce a fake vertical slice. |
| M7 Testing, security, performance | **Partial** | Corpus tests (accept, reject, every code), idempotence, golden output, property tests on random input, depth limits, mutation checks on the effect checker and formatter | Fuzzing in CI, sanitizers (no runtime yet), browser E2E, accessibility tests, benchmarks, performance regression gates. Benchmarks need a reference machine (open question Q22). |
| M8 Agent ecosystem | **Partial** | `skills/aqelith/SKILL.md` for the v0 CLI, with a test that fails if the skill omits a code or a command | MCP server (the directive says only after the CLI contract is stable; it now is for v0, but it has not been built), versioned machine-readable docs beyond the skill, agent productivity benchmarks |
| M9 Public developer preview | **Blocked** | Nothing | Brand approval (directive §0.2 gate). Public repository, packages, release notes and benchmark publication all wait for it. |

## Decisions taken (owner delegated these)

- ADR-0009: v0 syntax subset (answers Q6 and Q7).
- ADR-0010: no comments in v0; the formatter is a pure function of the tree (answers Q10).
- ADR-0011: root capability minting rule recorded; enforcement not implemented (answers Q23).

## Evidence

- `cargo test` in `compiler/`: 15 tests pass (8 corpus, 4 robustness, 3 skill).
- Mutation checks on a scratch copy: disabling the effect check fails 1 test; disabling formatter parentheses fails 2 tests.
- `cargo clippy --all-targets` after forced re-analysis: 0 warnings.
- CLI, run by hand: a rejected program exits 1; a missing file exits 2; an accepted program exits 0; `check --json` returns the code, message, span and line/column.

## What the v0 checker does NOT guarantee

- It reports the first syntax error only, and one type error per function.
- Effects are checked for calls only; there is no effect inference and no capability model yet.
- Integer arithmetic is not checked for overflow at run time; there is no run time.
- There is no interpreter or backend. "Accepted" means statically valid under v0 rules, nothing more.

## Risks carried forward

- The M6 gap is the largest. Everything the directive calls the "product" (web apps, resumability, accessibility) starts there.
- The v0 language is small enough that its decisions are cheap to change. That is intentional and recorded in ADR-0009.
- The CLI contract (exit codes, JSON shape) is now public inside the repository. Changing it is a breaking change for agents (ERROR_MODEL §6).

## Next steps, in order

1. Patch API over the IR, reusing the P1 precondition and atomicity rules, plus the formatter round trip (M5).
2. Fuzzing in CI and an interpreter for the v0 subset, so that "accepted" has a runtime meaning (M7 and M4 follow-up).
3. Renderer design decision and a browser harness, then the first web vertical slice (M6). This needs a decision on the view-tree format first.
4. M9 waits for the brand gate.
