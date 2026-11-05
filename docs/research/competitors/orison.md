# Competitor audit: Orison (Eldergenix/Orison)

Audit date: 2026-11-05. Auditor: AQELITH research engineer (static source audit first; the build and test suite were executed afterwards in a second pass, see "Commands executed" item 11).

## Sources

- Repository (authoritative candidate): https://github.com/Eldergenix/Orison
- Local clone (untrusted, read-only): `<local-disk>/aqelith-lab/orison@4b11ca8`
- In-repo docs read: `README.md`, `SECURITY.md`, `STABILITY.md`, `GOAL.md`, `BENCHMARKS.md`, `docs/ROADMAP.md`, `docs/compiler/AGENT_CONTEXT_ABI.md`, `docs/compiler/SELF_HOSTING.md`
- GitHub API (repo metadata, issues, PRs, releases, tags, owner): `gh api repos/Eldergenix/Orison[...]`
- Rejected candidates (see Identity verification): https://github.com/OrisonLang/orisonc, https://github.com/qbitedgetechnologies-prog/Orison, https://github.com/soraopenai14-byte/OrisonLanguage, https://github.com/megazear7/orison, https://github.com/LumenStorm/OrisonSpace
- Independent third-party coverage: none found.

## Identity verification

Searches (2026-11-05):
1. WebSearch "Orison programming language capability security effects contracts typed UI full-stack github": no Orison hit at all.
2. `gh search repos orison` (20 results): collisions listed below.
3. `gh search repos "orison language"`, "orison capability effects typed ui language", "orison-lang", "orison compiler agent diagnostics", "orison full-stack language contracts": nothing new.
4. WebSearch `"Orison" programming language "capabilities" "effects" "contracts" AI agents`: surfaced `Eldergenix/Orison`.

Candidates:
| Candidate | Verdict | Why |
|---|---|---|
| **Eldergenix/Orison** | ACCEPTED | README: "memory-safe, statically typed, compiled full-stack application language with ... capability-secured effects, JSON-first diagnostics, structural patching ... Agent Context ABI" (README.md:3-6). Matches every attribute in the brief (capability effects, typed `view` UI, `service`/route backend, agent diagnostics). |
| OrisonLang/orisonc | rejected | "Orison - An Elegant Systems Language with LLVM", C++23/CMake; org description "new systems language ... welcoming to Engineers". No capability/effect/UI/full-stack content in README or file tree (ORISON_SPEC.md exists but systems-language framing). A different language with the same name. Not read in depth: noted as a collision risk. |
| qbitedgetechnologies-prog/Orison | rejected | Static website for "Orison International General Trading" (Dubai phones shop). |
| soraopenai14-byte/OrisonLanguage | rejected | README is only "Orison Official Site"; CNAME `orisonlanguagedownload.cf`; 1 KB, no code. Suspicious/unrelated. |
| megazear7/orison | rejected | lit-html static site generator. |
| LumenStorm/OrisonSpace and others | rejected | Unrelated (stories/games/sites). |

Caveat: identity is established by self-description plus feature match only. The repo is a 1-star, 0-fork personal-account project (owner Eldergenix, a User account created 2021-09-01, bio "Building frontier agentic pipelines"), no website, no organisation. There is no independent confirmation that "the" Orison the brief refers to is this one; it is the only public match to the full feature list. Confidence in identity: medium-high.

## Inspected version

- Commit `4b11ca8c...` (`4b11ca8`, 2026-05-17T08:11:03-04:00) "feat(goal-v2): wave 5 (Layer 1+2+3) — 20 agents end-to-end". HEAD of default branch as cloned 2026-11-05.
- Workspace version 0.1.1 (`Cargo.toml`), `ori doctor` reports "bootstrap". No releases, no tags, 0 issues, 0 PRs (GitHub API). 9 commits, all between 2026-05-16 and 2026-05-17 (the whole history is ~1 day; commit messages cite "30-agent build-out" and "20 agents end-to-end", i.e. LLM-agent generated). Last push 2026-05-17T12:11Z, ~5.5 months before audit.
- Size: ~150 `.rs` files (~128k lines incl. tests per `wc`; README claims "~32,000 LOC"), 111 `.ori` files, 34 JSON schemas, 1270 `#[test]` attributes (README claims 456 passing).
- License: `LICENSE` is MIT (matches GitHub); README badge says Apache-2.0 (inconsistent).

## Commands executed

All results verbatim-summarised.

1. `gh search repos orison --limit 20 ...` -> list above.
2. `gh api repos/Eldergenix/Orison --jq ...` -> `["A secure, fast, agent-native programming language ...",null,"2026-05-17T01:51:25Z","2026-05-17T12:11:26Z",1,0,0,"MIT","Rust","User",1336]` (desc, homepage, created, pushed, stars, forks, open issues, license, language, owner type, size KB).
3. `gh api 'repos/Eldergenix/Orison/{issues,pulls}?state=all' --jq length` -> `0`, `0`; `/releases`, `/tags` -> `0`, `0`.
4. `git clone https://github.com/Eldergenix/Orison.git` into `<local-disk>/aqelith-lab/` then renamed to `orison@4b11ca8`. OK.
5. Build prep copy: `rsync -a --exclude .git <clone>/ <scratchpad>/work/` (build runs on a copy, not the clone).
6. FETCH attempt 1: `docker run --rm -v work:/src -v cargohome:/usr/local/cargo/registry -w /src rust:1.92 cargo fetch --locked` (network on). Result: `error: component download failed for rustc-aarch64-unknown-linux-gnu: unable to sync download to disk: cleaning up cached downloads: Input/output error (os error 5)`. Cause (observed): `rust-toolchain.toml` says `channel = "stable"` (NOT 1.92 as README.md:93 claims) with components rustfmt+clippy, so rustup tried to download a toolchain inside the container and hit an I/O error.
7. FETCH attempt 2: same with `-e RUSTUP_TOOLCHAIN=1.92.0` to bypass the file. Output: only `EOF`. Docker daemon stopped answering.
8. `docker info`, `docker ps`, `docker images`, `docker version` thereafter: `docker info` prints client section only; `docker version --format '{{.Server.Version}}'` prints `EOF`; some calls hang >30 s. `com.docker.backend` process alive; host root volume had 2.1 GiB free (85% used), a plausible cause of the I/O error and daemon failure (not confirmed). I did not restart Docker Desktop (shared host app, out of audit scope).
9. **Outcome: the project was never built or run. No test, benchmark, or CLI invocation was executed by me.** Nothing was run on the host. All implementation findings below are from static reading of source, tests, and docs. Test-count and benchmark claims are therefore NOT reproduced.
10. Static commands: `grep`, `wc`, `sed` over the clone (results cited with paths below).
11. **Second pass (2026-11-05, after Docker was moved to the SSD and restarted; supersedes items 6-9 for build/test outcome).** Build and tests ran on a copy of the clone in `aqelith-lab/build/`, with `rust-toolchain.toml` removed from the copy only, using the image's own Rust (`rustc 1.99.1`, image `rust:1-slim`, not the 1.92 the README claims). `cargo fetch --locked` ran with network on; build and test ran with `--network=none --memory=3g`, `CARGO_BUILD_JOBS=2`.
    - `cargo build --release --offline --locked`: exit 0, 35 s, 5 workspace crates compiled (`ori-compiler`, `ori-agent`, `ori-lsp`, `ori-pkg`, `ori`).
    - `cargo test --release --offline --locked`: exit 0, 51 test suites, 1097 passed, 0 failed, 0 ignored (summed from the `test result` lines). Wall time 105 s.
    - Not run: the project's benchmark harness, the CLI (`ori`) on the examples, the project's own "full gate" script.
    - A first attempt in this pass failed for harness reasons (a `time` call missing in `sh`; then rustup trying to re-download the toolchain without network). Both were fixed in the script, not in the project.

## Feature audit

Classification key: IT = Implemented and tested; IE = Implemented but experimental; PI = Partially implemented; PL = Planned; MC = Marketing claim only; NV = Not verifiable. "Tested" is judged from presence of unit/integration tests in source, not from a run (see build outcome). Therefore IT means "implementation and tests exist in the tree", not "tests pass". ABS = Absent: verified not present and not claimed by the project; used only for comparison rows and outside the six mandated classes.

| Feature | Classification | Evidence |
|---|---|---|
| Effect declaration (`uses a, b`) in function signatures | IT | README.md:233-249; effect list `crates/ori-compiler/src/effects.rs` (41 lines) |
| Transitive effect propagation + E0420 with Patch-IR `change_signature` fix | IT (14 unit tests) | `crates/ori-compiler/src/effect_propagate.rs:21,366`; test count grep. Depends on a body parser that covers calls only (README.md:690-697 lists supported constructs) |
| Package capability policy (`[capabilities].declared`), E0410 undeclared effect, `ori capability --json` manifest | IT | `effect_check.rs:73,121,168`; SECURITY.md:5-30 |
| Dependency capability audit (AUD0001/AUD0002) | IE | SECURITY.md:17-22 `crates/ori-pkg/src/audit.rs`; not inspected in depth |
| Runtime capability tokens (principal, expiry, deny-veto, audit list; CAP0001-0005) | IE | `capability_runtime.rs:1-30,136-176`; interpreter hook `interp_exec.rs:684-700` (guard on every call to a function with `uses`); `tests/capability_runtime_denial.rs` (3 tests). Only enforced in the tree-walking interpreter when a `CapabilitySet` is supplied; tokens are plain strings, unsigned |
| Runtime capability enforcement as a language/OS-level sandbox | MC | SECURITY.md:43 says "no capability runtime gating; the bootstrap is static-analysis-only" and "no real sandboxing"; contradicted by `capability_runtime.rs`; docs/ROADMAP.md:27 lists M35 "planned" while the code exists. Docs stale/inconsistent. Effects are not tied to actual syscalls (I/O is not implemented) |
| Capabilities as unforgeable object references (object-capability model) | ABS | Orison uses effect-name strings + package allow-list, not capability values passed as arguments |
| Contracts = versioned JSON output schemas (34) | IT | `schemas/*.schema.json` (34 files), `crates/ori-cli/src/schema_check.rs` (1208 lines), `crates/ori-cli/tests/schema_validation.rs`, STABILITY.md Tier 1 |
| Contracts = design-by-contract (pre/post conditions, invariants) | ABS | No `requires/ensures/invariant` in parser, AST or docs (grep over `parser.rs`, `ast.rs`, `body.rs`, docs/language: only English-prose hits). README "contracts" means schemas |
| Typed UI: `view Name(props: T) -> Html uses ui` DSL | PI | README.md:285-305; `ui_check.rs:1-8` describes itself as "Minimal UI manifest extraction ... label heuristic only"; `ui_render.rs` (1216 lines, 17 tests) renders a deterministic tree + diff. Views are checked at signature level; there is no browser/DOM runtime. Props are typed by the signature type checker, not by a dedicated UI type system |
| Capability-gated view (`uses ui, auth` prevents mounting without auth) | PI | README.md:304; enforced only as effect propagation (a caller without `auth` fails E0420), no mount runtime |
| A11y findings, design-token enforcement | IE | `ui_check.rs`, `design_tokens.rs`; README.md:306 |
| Native/mobile/desktop UI targets | PI | `mobile*.rs`, `desktop.rs`, `bundler.rs`; ROADMAP.md:23 "partial"; no evidence of runnable app |
| Full-stack: `service`/route + OpenAPI 3.1 extraction | IT (static) | README.md:251-283; `openapi.rs`; `ori openapi` |
| Real HTTP/WebSocket/queue runtime, `ori serve` | PI | `cmd_serve` supports only `--dry-run` (`ori-cli/src/main.rs:2102-2106`); `net_io.rs` has a TCP worker-pool prototype; README.md:704 "modules are declarations only"; stdlib `http.ori`/`websocket.ori`/`queue.ori` are declaration files (stdlib/std total 344 lines) |
| Typed SQL queries + migrations (shape check, toposort) | PI | README.md:307-322; `sql_check.rs`, `migration_graph.rs`; no database driver |
| Memory safety / borrow checker | PI (prototype) | README.md:677 "Borrow checker prototype B0010-B0050"; `borrow.rs`, `borrow_regions.rs`; README.md:690 region inference listed as "not yet" (ROADMAP: M23 planned) |
| Type system: no null, Option/Result, newtypes, exhaustive match | PI | `type_check.rs`, `type_infer.rs`, `exhaustive.rs`; README admits full inference in arbitrary bodies is not done |
| Interpreter (HIR/MIR tree-walker) | IE | `interp_exec.rs` (3009 lines); `ori run` |
| Native codegen (LLVM/x64) | PL / PI | README.md:699 "textual IR scaffold only"; `codegen_x64.rs`, `codegen_text.rs` exist; no executable output verified |
| WebAssembly output | IE | `wasm_encoder.rs`; "39-byte hello-module"; component/WIT emit files exist; execution on a wasm engine not verified |
| Compiler/runtime separation | PI | `ori-compiler` crate holds lexer through interpreter and net runtime in one crate; no distinct runtime package, no compiled-runtime ABI. See Architecture |
| Agent diagnostics: JSON diagnostics + 34 schemas, stable IDs | IT | `diagnostic.rs`, `schemas/diagnostic.schema.json`, `docs/diagnostics/INDEX.md`, test `diag_index_freshness.rs` |
| Stable node IDs, Patch IR (`ori.patch.v1`), dry-run, partial apply | IT | README.md:477-510; `patch.rs`, `patch_apply.rs`, `node_id.rs`; STABILITY.md Tier 1 |
| Budgeted context (`ori agent map --budget`, capsule, explain, diagnose, tests --affected) | IE | `crates/ori-agent`; `ori-agent/tests/agent_map.rs`; AGENT_CONTEXT_ABI.md still lists `diagnose`/`tests --affected` under "Planned" while README and CLI dispatch (`main.rs:279-286`) ship them: doc drift |
| Model-in-loop telemetry | IE | `ori agent telemetry` dispatch `main.rs:286`; `schemas/model-loop-telemetry.schema.json`; ROADMAP: M33 planned |
| LSP (hover, completion, rename, code actions, refs, etc.) | IE | `crates/ori-lsp` (many flow tests, e.g. `completion_flow.rs`); not exercised |
| Package manager (manifest, lockfile, SBOM, provenance, registry) | PI | `ori-pkg`; checksum is FNV-1a (lockfile.rs:15), local registry stub only; signing "unverified" (SECURITY.md:23-28) |
| Standard library (27 modules claimed) | PI | `stdlib/std/*.ori` are mostly signatures; total 344 lines for std; ROADMAP M27 in-flight |
| Self-hosting | PL | `docs/compiler/SELF_HOSTING.md:5`: "Design only. No implementation work has been done"; `compiler/stage1/{lexer,parser}.ori` exist as stage-1 experiments |
| "Sub-100 µs edit-check-repair round trip" | MC (misleading) | README.md:5,73,373-403; see Performance evidence |
| "456 passing tests, 0 failing" / full gate green | IT (suite), figure stale | `cargo test --release` ran here: 1097 passed, 0 failed across 51 suites, so the suite is green but the "456" figure is out of date. The tree has 1270 `#[test]` attributes versus 1097 executed tests; the 173 difference was not investigated. The project's own "full gate" script was not run |
| Production readiness / 1.0 | PL | STABILITY.md "alpha ... until 1.0 breaking changes permitted"; GOAL.md:1081 defines production criteria |

## Architecture notes

- Rust workspace (`Cargo.toml`): `ori-compiler` (lexer, error-tolerant CST with node IDs, parser, resolver, type check/infer, effects, borrow, patch IR, HIR/MIR, interpreter, async runtime, wasm/LLVM-text/x64 codegen, UI/OpenAPI/SQL extraction, net_io), `ori-agent` (capsule/map/symbols), `ori-cli` (30+ subcommands in one 3501-line `main.rs`), `ori-lsp`, `ori-pkg`; also `ori-docsite`. Only `serde`/`serde_json` dependencies (`crates/ori-compiler/Cargo.toml`). README claims "Five crates, ~32,000 LOC"; actual tree is larger (6 crates, ~128k lines incl. tests).
- Compiler/runtime separation is weak: the "runtime" (interpreter, async scheduler, net_io, ui_render, capability_runtime) is a module inside the compiler crate. Compile-time effect analysis and run-time token guard are two independent mechanisms joined only by `def.effects`.
- Source language: indentation-based, `module`, `type X wraps Str` newtypes, records, variants, `fn f(..) -> T uses e1, e2:`, `service`, `view`, `migration`, `query`. Many example functions are signature-only (e.g. `examples/todo_app/src/api.ori`), i.e. contracts for a body that does not exist; the body parser handles only a limited set of constructs.
- Design idea worth noting: everything is a schema-versioned JSON envelope (`ori.<thing>.vN`), so the toolchain is an API for agents; stability tiers (STABILITY.md) bind schema names, diagnostic IDs, CLI flags, Patch IR ops.
- Effects are a closed set of strings (`fs.read`, `net.outbound`, `db.write`, `auth`, `ui`, `gpu`, ...) plus user-declared uppercase names; no effect polymorphism, handlers, or row types seen.

## Security considerations

- Static layer is real but shallow: effect names declared per function, allowed per package, propagated across the call graph. It rejects missing declarations; it cannot see ambient authority because no I/O primitives exist in the runnable language yet ("no real network stack at all", SECURITY.md:41).
- Runtime layer (`capability_runtime.rs`): token lookup by effect name, principal binding (`issued_to`), expiry, deny-veto, audit list. Tokens are in-memory structs with no signature/attenuation/delegation. `ori capability check --dry-run` builds the token set from a `--has` CLI list (`main.rs:1770-1790`), i.e. it is a simulator, not an enforced boundary. Guard only fires in the interpreter path.
- Supply chain: lockfile checksum is FNV-1a (non-cryptographic) by the project's own admission (SECURITY.md:36; lockfile.rs:15). No sandbox for dependencies. Provenance "verification" is string matching on an unrecognised signature field.
- Memory-safety claim for compiled programs is unsubstantiated: borrow checker is a prototype and no native backend exists. Compiler itself claims zero `unsafe` Rust, enforced by `tests/unsafe_surface_report.rs`.
- SECURITY.md has no disclosure channel (points at bug-report template) and is stale relative to the code (says no runtime gating).
- Provenance of the repo itself: produced by LLM agents in ~1 day by a single account, no review history, no external contributors; treat claims accordingly.

## Performance evidence

- Source of numbers: README.md:373-403, `BENCHMARKS.md`, `BENCHMARKS.results.json`; harness `crates/ori-compiler/src/bench.rs`. Platform stated: Apple Silicon, rustc 1.92, n=100. NOT reproduced by me: the build now works (35 s release build) but I did not run the benchmark harness, and this machine (M1, 8 GiB, Docker VM) is not comparable to the stated one.
- Published figures: cold check mean 4.0 µs / p50 2.2 µs; warm check p50 20.2 µs; patch validate p50 1.5 µs; patch apply p50 9.8 µs; composite "~77 µs". BENCHMARKS.md gives different numbers for the same suites (e.g. cold p50 ~4.4 µs, warm mean ~34.6 µs vs README 24.1 µs): the two documents disagree.
- Validity problem (verified in source): benchmark inputs are tiny fixtures, e.g. `DEFAULT_MEDIUM` (`bench.rs:1034-1045`) is an 11-line, signature-only module. The figures measure parsing of ~10 lines; they say nothing about real programs, nothing about type inference over bodies, and nothing about codegen/execution. "Sub-100 µs round trip" is a micro-benchmark artefact. The word "real measured numbers" is accurate only for that fixture.
- No comparison against other tools, no memory numbers, no runtime/app performance data.

## Known limitations

Stated by the project (README.md:664-710 "Honest scope", ROADMAP.md, SECURITY.md:36-47):
- No full expression-level type inference; no region-inference borrow checker; no optimising native codegen (textual IR only); single-threaded cooperative async; non-cryptographic registry checksums; no real HTTP/WebSocket/queue runtime; no self-hosting.

Observed by this audit:
- Docs contradict each other and the code (SECURITY.md vs capability_runtime.rs; ROADMAP M28/M29/M35 "planned" vs modules present; AGENT_CONTEXT_ABI "planned" vs shipped; README benchmark table vs BENCHMARKS.md; README Apache-2.0 badge vs MIT LICENSE; README "pinned Rust 1.92" vs `channel = "stable"`).
- Many shipped components are scaffolds: `ori serve` is dry-run only; examples are mostly signature-only; stdlib is declarations.
- Single author, 9 commits in one day, 0 issues/PRs/releases/tags, 1 star, dormant ~5.5 months. Bus factor 1; no evidence of real users.
- No design-by-contract in the language despite "contracts" branding (they are JSON schemas).
- Name collision risk: `OrisonLang/orisonc` (C++/LLVM systems language, active as of 2026-11-05) is a different project.
- Build and unit tests verified in a second pass (see "Commands executed" item 11). Still not verified: benchmarks, CLI behavior on the examples, and the project's own "full gate".

## Ideas worth adopting for AQELITH

1. Schema-versioned envelope for every tool output with a Tier-1 stability policy (schema names, diagnostic IDs, flag names immutable; STABILITY.md). Strong fit for agent-facing tooling.
2. Stable structural node IDs + Patch IR with per-op partial apply and `dry-run` preview instead of text diffs (README.md:477-510).
3. Diagnostics that carry machine-applicable fixes expressed in the patch format (E0420 -> `change_signature`), so agents repair without re-planning.
4. Budgeted context packing (`agent map --budget N` with explicit `truncated` flag) and per-symbol "explain" cards: cheap, measurable agent ergonomics.
5. Capability manifest emitted by the compiler plus a dependency capability diff in audit (AUD0001/2): a good supply-chain primitive. AQELITH should do better by using unforgeable capability values and signed attenuation, and by actually tying effects to I/O primitives.
6. Cautionary lessons: do not publish micro-benchmarks on 10-line fixtures as headline claims; keep docs generated from code/CI to avoid the drift seen here; separate "shipped" from "scaffold" status in a machine-readable feature table; keep a runtime package distinct from the compiler.

## Confidence

| Section | Confidence | Why |
|---|---|---|
| Sources | high | URLs fetched through `gh`; repo metadata cross-checked |
| Identity verification | medium | Only public match to the full feature set, self-described; no independent corroboration; name collisions exist |
| Inspected version | high | Commit hash, dates and counts taken from git/API |
| Commands executed | high | Failures recorded as observed; first pass could not build, second pass built and ran the test suite (item 11) |
| Feature audit | medium | Static reading only; not run; "tested" = tests exist in tree. Capability-runtime, effect propagation and UI claims checked at source level, other rows partly via file names/docs |
| Architecture notes | medium | Based on README, crate layout, headers; modules not read exhaustively |
| Security considerations | medium | Core guard code and SECURITY.md read; audit/provenance code not read in depth |
| Performance evidence | medium | Harness and fixture inspected; numbers unreproduced |
| Known limitations | high for self-reported items, medium for observed |
| Ideas | medium | Judgement calls grounded in the evidence above |
