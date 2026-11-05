# Competitive Analysis (M1)

Inspection date for every project below: 2026-11-05. This document synthesizes the per-project audits in `docs/research/competitors/`; each cell cites the classification used there. Nothing here is an AQELITH claim.

Classification codes: **IT** implemented and tested; **IE** implemented but experimental; **PI** partially implemented; **PL** planned; **MC** marketing claim only; **NV** not verifiable; **ABS** verified absent and not claimed (comparison rows only, outside the six mandated classes).

## Versions inspected

| Project | Source | Version inspected | Audit |
|---|---|---|---|
| Zerolang | github.com/vercel-labs/zerolang | commit `7e1a64d2` (2026-09-16), `zero 0.3.4` | [zerolang.md](../research/competitors/zerolang.md) |
| Astra | github.com/MAJIDNISAR/astra-lang.org (docs site only; compiler source private) | commit `ca24fb9` | [astra.md](../research/competitors/astra.md) |
| Naze | github.com/jasonmamy/naze-lang | commit `9e3b301` (2026-03-27) | [naze.md](../research/competitors/naze.md) |
| Orison | github.com/Eldergenix/Orison (identity by self-description only) | commit `4b11ca8`, workspace 0.1.1 | [orison.md](../research/competitors/orison.md) |
| AICore | github.com/keaz/aicore | commit `8976f47` (2026-10-11), 0.1.0 | [aicore.md](../research/competitors/aicore.md) |
| Edict | github.com/Sowiedu/Edict | commit `2dbed48` (2026-10-25), v1.23.2 | [edict.md](../research/competitors/edict.md) |

## What I ran locally (all in Docker, one M1 machine, 4 GiB VM)

| Project | Build | Tests | Not run |
|---|---|---|---|
| Zerolang | official `make` (-Os) ok, 103 s | native script **not completed** on any platform (AArch64 fixture failure; x64 stopped silently) | conformance, bench, evals |
| Astra | none (no source) | none | everything native; only a public WASM build was exercised |
| Naze | default members ok, 251 s | 465 passed, 0 failed | WASM runtime, playground, CLI on examples |
| Orison | ok, 35 s | 1097 passed, 0 failed | benchmarks, CLI on examples |
| AICore | ok, 119 s | 1342 passed, 5 failed, 2 ignored (3 failures unexplained) | CLI on examples, benchmarks |
| Edict | ok, 16 s after `npm ci` | 2674 passed, 1 skipped | MCP server end to end, deploy, benchmarks |

Build times are not comparable across rows (different languages, a VM-limited 2-job build, partly emulated). They are recorded only as observations.

## Matrix

| Dimension | Zerolang | Astra | Naze | Orison | AICore | Edict |
|---|---|---|---|---|---|---|
| **Language model** | Graph is compiler input (IT); `.0` text is a projection (IT); memory-safety model IE | Indentation syntax, formatter to canonical form (IE); language feature set PI | Declarative UI DSL, PEG grammar (~157 rules, LL(1) claim NV) (IT) | `uses` effects (IT); no-null/Option/Result/exhaustive match PI | AST→IR→typecheck→LLVM text→clang (IT); ownership PI, no lifetimes (PL) | JSON AST, no text syntax (ABS for readable syntax); bidirectional type checker (IT); refinement types via Z3 |
| **Agent DX** | Graph queries, typed patches, dry-run, embedded versioned skills (IT) | JSON diagnostics with stable codes (IE); `astra fix` inconsistent (IE) | `nazec context` (IE), JSON diagnostics (IT), whole-grammar export (IT); per-tier export MC | 34 versioned JSON schemas, stable node IDs, Patch IR (IT); budgeted context (IE) | Protocol-version negotiation, daemon JSON-RPC, patch/checkpoint (IE); SARIF + `explain` (IT) | 22 MCP tools, node-id patches, structured errors with candidates (IT) |
| **Compiler** | One C11 program (~128k lines); direct ELF/Mach-O/COFF emitters (IE) + LLVM IR (IE) | Source private (NV); LLVM IR backend IE | Rust workspace; compiler-time component inlining (IT) | Rust; native codegen is a text scaffold (PL/PI) | Rust; incremental not assessed; self-hosted compiler IE | TypeScript; incremental checking (IT); own WASM encoder (IT) |
| **Runtime** | Native, no libc dependency on default backends (IE) | Interpreter (IE); native parity NV | WASM runtime + Canvas2D (IE); native winit/tiny-skia (PI) | Tree-walking interpreter (IE); WASM output (IE) | C runtime (~30.8k lines) linked via clang (IT); async runtime (IE) | WASM run in Node with limits (IT); pure-JS interpreter (IE) |
| **Performance** | Claims NV; no committed benchmarks found | Author-only numbers NV; benchmark page admits a harness bug | Token/size claims NV; wasm blob 404,788 B vs advertised 395 KB | "Sub-100 µs" measures 11-line fixtures (MC) | Perf gate PI; no numbers run by me | Size figures unreconciled with README; no speed claims verified |
| **Security** | Explicit `World` capabilities IE; SIGSEGV on deep input reproduced (issue #435) | `@pure` effect lint only; capability model PL | Hydration JSON written unescaped into a `<script>` tag (read, not exploited) | Runtime capability tokens are in-memory strings (IE); no object-capability model (ABS) | Fixed 8-effect set; `extern "C"` bypasses tracking (MC); free(): invalid pointer in one TLS test (unexplained) | Lexical-only file sandbox; HTTP open unless allow-listed; child process inherits env |
| **UI** | Not covered by the audit | Not covered | Canvas2D + hidden ARIA mirror (PI); no event listeners on the mirror | Typed UI at signature level (PI) | Not covered | Not covered |
| **Web** | Not covered | WASM build of compiler IE; wasm32-wasi target PI | SSR/SSG emits HTML, hidden once WASM starts (IE); SEO meta (IT) | OpenAPI extraction (IT); `ori serve` dry-run only (PI) | HTTP server tests exist; web target not assessed | Browser bundles; Cloudflare deploy (IE, not exercised) |
| **Native** | ELF/Mach-O/COFF direct emitters x64+aarch64 (IE); AArch64 backend rejected a conformance fixture | Native pipeline NV | Desktop Linux only; Android is a WebView wrapper; no iOS | Native codegen not real (PL/PI) | Native via clang (IT) | Not a native language |
| **Compatibility** | Targets PI; fixture runs on x86_64 but not AArch64 host | Claimed 4 OS builds NV | Per-feature, see Native/Web | WASM IE | Native via clang; platform matrix not assessed | Node/Browser/Deno/Cloudflare adapters (IE) |
| **Tooling** | CLI IT; debugger not assessed | `fmt` IE; LSP PL; MCP NV | LSP, VS Code ext., playground (IE) | LSP IE | Formatter (IT), LSP/DAP (IE), test harness with property tests (IT) | Lint (IT); MCP (IT) |
| **Ecosystem** | Local-path package manager PI; 34 stdlib modules IE | Mifa package manager NV | Registry IE; discovery network IE | Manifest, lockfile, SBOM PI; stdlib PI | Lockfile + checksums + SBOM IE | npm package; schema migration (IT) |
| **Maturity** | Single author, v0.3.4, issues open on identity (#348) | Source private; docs site only | 2 stars, 0 releases, 0 tags; says it is a prototype | 9 commits in 2 days, 1 star, no releases | 0.1.0, 1 committer, no releases, mostly written with ChatGPT Codex per its README | 13 stars, single maintainer, v1.23.2 |
| **Limitations** | Patch/check accept type-invalid literals (reproduced on -Os); stable ids are position-derived | Closed source; benchmarks unreproducible | A11y mirror rebuilt each pass; claims ahead of code (tier export, three outputs) | Docs contradict code; no DbC; "456 tests" stale (1097 pass) | Heap never freed; IR output-only; check/build parity fixed at HEAD but issues open | Units don't compose (spec says they do); mock-repairer "benchmark"; no readable syntax |

## Notes on reading the matrix

- A cell states what the audit classified or observed; it is not a ranking. The audits used different evidence depth (see the "What I ran locally" table), so equal-looking cells are not equally verified.
- "Not covered" means the audit did not examine that dimension, not that the project lacks it.
- Gaps recorded in the audits and not resolved: Astra's compiler source is unavailable; Zerolang's test suite was not completed; three AICore test failures (UDP/DNS, TLS free(): invalid pointer, selfhost driver) are unexplained; the Orison repository's identity rests on self-description.
