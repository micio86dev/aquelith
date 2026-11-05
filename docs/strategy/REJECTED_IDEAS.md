# Rejected Ideas (M1)

Ideas seen in the audited projects (`docs/research/competitors/`) that AQELITH should not adopt, with the evidence. A rejection is about the idea as implemented there, not a judgement of the project.

| # | Idea | Source | Why rejected |
|---|---|---|---|
| 1 | Programs exist only as a JSON AST, with no human-readable syntax | edict (readable syntax: ABS) | The directive requires human-readable source alongside the machine form. Edict's own audit lists the missing lossless projection as its largest risk. |
| 2 | Graph as the sole authority, text as a derived projection with sync commands | zerolang (IT, with a reconciliation layer) | Not rejected outright (see D1), but this authority split is rejected as-is: the audit reproduced a case where only the text path caught a type error the graph path accepted. Any AQELITH design needs one declared authority and a verified derivation. |
| 3 | Canvas2D as the only web renderer; hiding the DOM | naze (PI accessibility) | Directive §8.3 forbids it. Observed: ARIA mirror has no event listeners, is rebuilt every render pass, and static HTML is hidden with `display:none` after WASM starts (a cloaking and accessibility risk). |
| 4 | Position-derived 32-bit node ids and name-derived symbol ids as "stable identity" | zerolang (PI) | Not rename-stable; Zerolang's own issue #348 on stable node identity is open. Use persistent opaque ids. |
| 5 | Non-cryptographic hashes for graph and artifact identity | zerolang | Unsafe if signatures or multi-agent merge are planned. |
| 6 | Reporting `ok` while the build-readiness report says not buildable | zerolang (reproduced: `check` ok, `check --json` `buildable:false`) | Two verdicts for one program undermine agent loops. One authoritative verdict. |
| 7 | Calling an effect lint a capability model | astra (capability model PL) | Capabilities need values and enforcement; a `@pure` check is only an effect restriction. |
| 8 | Runtime capability tokens as in-memory strings matched by effect name | orison (IE) | Forgeable by construction; SECURITY.md in the same repo still says there is no runtime gating (documentation drift). |
| 9 | `capabilities` and `effects` as two clauses over the same fixed token set | aicore (capabilities clause IE) | Duplicated concept. Choose one mechanism and make capabilities values. |
| 10 | Fixed effect taxonomy without effect polymorphism or user-declared effects | aicore (PL for those) | Blocks library authors; `extern "C"` also bypasses tracking (MC). |
| 11 | Lexical-only path sandbox, HTTP open by default, child process inheriting the full environment | edict (sandbox claim MC partly) | Weak defaults. Real-path checks, default-deny network, scrubbed environment. |
| 12 | Author-assigned string node ids | edict (IT) | Collision and reuse risk; derive from content or assign persistent opaque ids. |
| 13 | Headline micro-benchmarks on tiny fixtures ("sub-100 µs" on an 11-line module) | orison (MC) | Measures parsing of ~10 lines. Directive §7.5: budgets state hardware, dataset, runs, variance. |
| 14 | A "benchmark" scored by a hard-coded mock repairer | edict (IE) | Says nothing about LLM authoring success. |
| 15 | Publishing a roadmap as proof of capability while the compiler source is closed | astra | Nothing to reproduce; contradicts the evidence-first rule. |
| 16 | Shipping an MCP server and a fix engine before basic lexing gaps are closed | astra (`n-1` without spaces rejected) | Priorities inverted. MCP waits for a stable CLI. |
| 17 | Marketing claims ahead of code (per-tier grammar export, three outputs from one source, pixel-identical rendering, "production-ready") | naze (MC / PL) | AQELITH's docs describe implemented behavior only. |
| 18 | Parsers without recursion or size limits | zerolang (SIGSEGV reproduced, issue #435) | Fuzzing and depth limits from day one. |
| 19 | Heap memory never reclaimed | aicore (heap reclamation PL) | Contradicts the memory-safety pillar. |
| 20 | Documentation numbers that drift from code | orison ("456 tests" vs 1097 passing; MIT vs Apache badge), naze (395 KB vs 404,788 B) | Generate docs facts from CI. |
