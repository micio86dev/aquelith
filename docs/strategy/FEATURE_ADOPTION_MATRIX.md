# Feature Adoption Matrix (M1)

Every row traces to the "Ideas worth adopting" and "Feature audit" sections of the audits in `docs/research/competitors/` (file named in the Source column). Decisions are provisional until M2 (architecture) and M3 (prototypes) produce evidence.

Decision codes: **A** adopt directly as a general concept; **A+** adopt with architectural improvements; **R** reject (see [REJECTED_IDEAS.md](REJECTED_IDEAS.md)); **D** defer until evidence justifies it.

| # | Feature | Source | Decision | Rationale |
|---|---|---|---|---|
| 1 | Optimistic-concurrency preconditions on every mutation (graph hash, node hash, field value), failure leaves the store byte-identical | zerolang (IT; verified byte-identical on rejection) | A+ | Stale-edit protection is a directive requirement. Improve: use a collision-resistant hash and tie preconditions to persistent ids, not position-derived ones. |
| 2 | Patch dry-run returning JSON with before/after hashes and per-op results | zerolang (IT), orison (IT) | A | Cheap, testable, directly serves "validate patches before applying". |
| 3 | Handle shorthand with nearest-match error on a miss | zerolang (IT) | A | Reduces agent token cost; low risk. Measure it. |
| 4 | Version-matched skills served by the binary | zerolang (IT) | A | Prevents skill/compiler drift. Matches directive §12.2. |
| 5 | Diagnostics as repair contracts: stable codes, expected/actual, `explain`, fix-safety levels | zerolang (IE), astra (IE), aicore (IT), edict (IT) | A+ | Four projects converge. Improve: line/column mandatory (Astra returns null for several codes), complete per-code explain text, one machine-readable fixability class per code. |
| 6 | Runtime codes separated from compile codes, JSON on stderr | astra (IE) | A | Keeps stdout as the program's. |
| 7 | Validation ladder (decoded, shape, resolved, typed, effect-checked, buildable) shown in status | zerolang (IE) | A+ | Improve: gate mutations on the strongest level; Zerolang's gate stops at shape (reproduced). |
| 8 | Stable ids | zerolang (PI), orison (IT), aicore (IT), edict (IT) | A+ | Needed for D1/D2. Improve: persistent opaque ids assigned at creation, carried through rename/move; separate content hash for staleness. Zerolang's position-derived ids are not rename-stable (R). |
| 9 | Node-id-addressed structural patch protocol (replace/delete/insert, atomic reject) | edict (IT), orison (IT) | A+ | Improve: content-hash ids rather than author-assigned strings (Edict). |
| 10 | Structural rewrite-by-example with dry-run listing every site | zerolang (IE) | D | Valuable for cross-cutting edits, but experimental upstream. Revisit after patch API exists. |
| 11 | Schema-versioned envelope for every tool output, stability policy | orison (IT), aicore (IT) | A | Required by the CLI contract in directive §11. |
| 12 | Protocol-version negotiation + daemon JSON-RPC mirror of the CLI | aicore (IE) | A+ | Do after the CLI contract is stable (directive §12.1: MCP waits). |
| 13 | MCP server with many tools | edict (IT, 22 tools) | D | Directive requires CLI first; MCP must not duplicate compiler logic. |
| 14 | Budgeted context packing with explicit `truncated` flag | orison (IE) | A | Cheap, measurable agent ergonomics. |
| 15 | Machine-applicable fix expressed in the patch format | orison (IT, E0420), astra (IE) | A+ | Improve: only where the fix is provably safe; never auto-apply ambiguous fixes (directive §11.1). Astra's `astra fix` was inconsistent. |
| 16 | Whole-grammar export for constrained decoding | naze (IT; per-tier MC) | A+ | Improve: per-subset export plus a CI check that the grammar accepts all examples and rejects fixtures. |
| 17 | `llms.txt` / condensed `AGENTS.md` shipped with the toolchain | naze | A | Cheap. Must be generated from, and tested against, the compiler version. |
| 18 | Compile-time component inlining and one-file-per-component | naze (IT) | D | The context-bounding claim ("sigma") is a hypothesis; measure before adopting. |
| 19 | SSR/SSG emitting real semantic HTML + meta/JSON-LD | naze (IE; SEO IT) | A+ | Improve: keep the HTML visible; Naze hides it after WASM starts. |
| 20 | Headless UI test runner using the same language, including accessibility assertions | naze | A | Fits directive §9.3 renderer conformance. |
| 21 | Effect inference with call-path provenance in errors | astra (PI), aicore (IT) | A+ | Improve: effect polymorphism and user-declared effects (AICore's fixed set is PL for these). |
| 22 | Package capability policy + dependency capability diff in audit | orison (IT / IE) | A+ | Strong supply-chain primitive. Improve with unforgeable capability values and signed attenuation (see D3). |
| 23 | Contract-driven optimization (elide a divide-by-zero guard when `requires` proves non-zero) | aicore (IE) | A | Small, demonstrable value of contracts. |
| 24 | Contracts verified by an SMT solver with timeouts, a cache, and explicit "undecidable" reporting | edict (IT) | D | Powerful, but Edict's own corpus proves 71.7% of contracts; defer until the type/effect core exists. |
| 25 | Portable built-in solver for the linear-integer fragment, Z3 optional | edict (IE) | D | Needs differential soundness testing against Z3 before trusting. |
| 26 | Deterministic record/replay of host effects | edict (IT) | D | Good for agent debugging; depends on the effect system design. |
| 27 | Approval gates that propagate through callers | edict (IT) | D | Interesting type-level human-in-the-loop idea; depends on D3. |
| 28 | Formatter proving the program unchanged (AST equality) | astra (IE) | A | Make the invariant tested, with idempotence property tests. |
| 29 | Compiler as a pure JSON function over WASM with one import | astra (IE) | D | Attractive sandbox boundary; revisit with the web runtime. |
| 30 | Failure-audit discipline: probe-based, with repro programs, "check passes implies build passes" parity rule | aicore | A | Process rule, zero cost. |
| 31 | Status as data generated from CI, evidence-linked | astra (status.json idea) | A+ | Improve: each row links a passing test, generated not hand-maintained (see D5). |
| 32 | Differential interpreter-vs-native fuzzing as the parity gate | astra, aicore (test harness IT) | A | Only when more than one backend exists. |
| 33 | Hermetic build manifest, reproducible builds | aicore (IT, compiler side) | A | Directive: deterministic builds where feasible. |
| 34 | Direct native emitters without LLVM | zerolang (IE) | D | Directive §7.4: no LLVM/Cranelift/custom backend commitment without benchmarks. |
| 35 | Hosted registry / discovery network / trust scores | naze (IE), astra (PL) | D | Out of scope until the language and package format exist. |

Rows 5, 8, 9, 11, 15 and 21 are the ones the architecture phase (M2) should treat as first-class requirements; the rest are inputs to M3 prototypes.
