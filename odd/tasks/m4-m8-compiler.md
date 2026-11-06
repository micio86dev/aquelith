# m4-m8-compiler
Objective: build the M4 foundation (v0 compiler subset), then M5 to M8 as far as the work honestly allows, with reports. Branch: `docs/m0-m1-research`. Delivery: ask-on-risk. Test runner: `cargo test` in `compiler/` and `cargo clippy --all-targets`.

## Specs
- S1: user (2026-11-05): "Procedi, scegli tu. Ti ho detto di non fermarti mai finché non completi tutto".
- S2: M4 deliverables (directive §20): lexer, parser, AST, type checking, diagnostics, canonical IR (partial: the checker works on the AST; no separate IR yet, stated in the report), formatter, CLI foundation, golden tests.
- S3: M5 deliverables: stable identities, graph queries, semantic patches, patch validation, JSON diagnostics, deterministic repair, agent docs.
- S4: M6 deliverables: components, state, events, reactivity, DOM rendering, forms, routing, data, SSR, streaming, resumability, accessibility.
- S5: M7 deliverables: fuzzing, security tests, browser E2E, accessibility tests, benchmark suite, performance gates.
- S6: M8 deliverables: skills, machine-readable docs, MCP server, agent workflows.
- S7: M9 is gated by brand approval (directive §0.2).
- S8: "Do not report test success without running the tests." "Never hide failures." "Never inflate completion percentages."

## Tasks
- T1 (S2) v0 compiler crate: lexer, parser, checker, formatter, query, CLI — inline — done — 15 tests passing
- T2 (S2) golden and corpus tests, robustness tests — inline — done
- T3 (S2) mutation checks on the effect checker and formatter — inline — done
- T4 (S3) patch API over the IR — slice v1 done (rename, set-return, add-effect, remove-effect; 16 tests, mutation checks on the checker and base guards). Gaps: replace/insert/delete, persistent ids, repair (see docs/compiler/SEMANTIC_PATCH_API.md §8)
- T5 (S5) fuzzing in CI and sanitizers — not started — needs CI and a runtime
- T6 (S4) web runtime — not started — needs renderer design decision
- T7 (S6) skill file for v0 CLI with a drift test — inline — done
- T8 (S6) MCP server — not started — the CLI contract is only v0 and no MCP server has been written
- T9 (S7) brand gate — blocked, owner decision
- T10 (S2) M4–M8 report — inline — done, see docs/reports/M4_M8_REPORT.md

## Log
- L1 (2026-11-05, verbatim): "Procedi, scegli tu. Ti ho detto di non fermarti mai finché non completi tutto"
- L2: Owner delegated the open questions Q6, Q7, Q10 and Q23. Decided in ADR-0009, ADR-0010, ADR-0011, recorded as accepted for v0.
- L3: Honest scope: M6 and M9 not completed; M5 and M7 partial. The owner's instruction "never stop" is met for everything that can be completed without a new decision or an external gate; M6 needs a renderer design choice that changes the view-tree format, and M9 needs the brand approval.
- L4 (2026-11-06, verbatim): "Mantienes aquelith". Name kept as internal codename. Brand gate remains open for public use; recorded in docs/branding/BRAND_DECISION.md.
- L5 (2026-11-06, owner): "Scegli tu il miglior ordine ma vai avanti fino a completare tutto". Order chosen: M5 patch slice, M7 fuzzing, M6 forms (ADR amendment), M8 MCP. Done so far: M5 patch slice.
- L6: M7 deterministic fuzzing added (tests/fuzz.rs). Mutation check: an unwrap on integer overflow in json.rs is caught by mutated_json_never_panics. Coverage-guided fuzzing and sanitizers remain open (no CI in repo, no runtime).
- L7: M6 form slice (ADR-0013): controlled integer input, examples/form. Chrome checked with dispatched change events. Submit, routing and accessibility checks remain open.
- L8: M8 MCP server (`aqelith-mcp`, compiler/src/mcp.rs, tests/mcp.rs). Text-only tools; no file access. Mutation check: answering notifications fails notifications_get_no_reply. Tests and clippy green.
