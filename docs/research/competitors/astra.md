# Competitor audit: Astra (astra-lang.org)

Scope: AQELITH competitor research. Everything below is either (a) observed first-hand in this audit, (b) quoted from the project's own public pages and labelled as a claim, or (c) marked Not verifiable. No benchmark number here was independently reproduced.

Bottom line: the Astra compiler source and native binaries are NOT public. The only public artifacts are the documentation website (public GitHub repo, static HTML), and a 1.1 MB WebAssembly build of the front end plus interpreter that ships inside that website repo. I audited the docs in full and exercised the WebAssembly module inside Docker with networking disabled. I could not build or run the native/LLVM path, the package manager, the MCP server, or any test suite.

## Sources

- https://astra-lang.org/ (home; JSON-LD says softwareVersion 0.13.0-alpha, license MIT, author Majid Nisar)
- Pages read in full (saved as text, then read): /start.html, /playground.html, /language.html, /agents.html, /mifa.html, /progress.html, /benchmarks.html, /vision.html, /about.html, /download.html
- https://astra-lang.org/about.html (states: "the source repository is private and opens at v1.0-beta (early access on request)")
- https://astra-lang.org/download.html (states: binaries go to testers through a private release page; public downloads "will be published here")
- https://github.com/MAJIDNISAR/astra-lang.org (public website repo, 44 commits, created 2026-07-06, 1 star, 0 open issues, no license file, no GitHub releases)
- https://github.com/MAJIDNISAR/Astra (the compiler repo named in the website README) returns HTTP 404 anonymously; api.github.com/repos/MAJIDNISAR/Astra also 404. Consistent with "private".
- https://github.com/majidnisar/astra and https://github.com/astra-lang/astra: HTTP 404.
- GitHub user repo listing for MAJIDNISAR (via API): no compiler/mifa repo among public repos; only `astra-lang.org`.
- https://majidnisar.com/projects/astra/ (author project page; only layout CSS extracted, no technical content read)
- Local copy: `<local-disk>/aqelith-lab/astra@ca24fb9/` (clone of the website repo)

Identity check: the project is Astra by Majid Nisar, "AI-first native programming language", LLVM-backed, with the Mifa package manager. Not to be confused with DataStax Astra (database) or other "Astra" languages. The repo identified as authoritative for the site is the website repo only; the real source is private.

## Inspected version

- Advertised release: v0.13.0-alpha, dated 2026-11-05 (download page, progress page, status.json).
- Inspected artifact: website repo commit `ca24fb9af807f53c33841da24ef8b9b3b47eab65` (2026-11-05T22:28:28+05:30, "content: v0.13.0-alpha: astra mcp, runtime codes, WebAssembly download, playground module 0.13.0").
- Inspected binary: `assets/playground/astra.wasm`, sha256 `e879388187ad440f2c166b7f8b5e02677de1d95fb193d37c9a78271407700e78`, WebAssembly MVP v1, 1,146,995 bytes. Its exports: `memory, pg_alloc, pg_call, pg_free, pg_result_len, __data_end, __heap_base`. Single import `env.pg_out`. It is labelled in `assets/playground/worker.js:3` as "built from the compiler's playground/ crate".
- Inspection date: 2026-11-05 (same day as the release; the project is under hours-old churn: v0.12.0, v0.12.1, v0.12.2, v0.13.0 were all released 2026-11-04 to 11-05 per the progress page).
- The wasm binary was not independently matched to a source commit (source is private), so its provenance is unproven.

## Commands executed

Host-side (fetch/read only; nothing from the downloaded repo was executed on the host):

1. `curl -sL https://astra-lang.org/<page>.html` for the 11 pages, converted to text with sed. Result: all fetched, 200.
2. `curl -s -o /dev/null -w '%{http_code}' https://github.com/MAJIDNISAR/Astra` -> `404`. Same for `majidnisar/astra`, `astra-lang/astra` -> `404`. `https://github.com/majidnisar` -> `200`.
3. `curl -s https://api.github.com/repos/majidnisar/astra-lang.org` -> public, not private, created 2026-07-06, 1 star, 0 open issues, `license: null`. `/releases` -> `[]`.
4. `git clone https://github.com/MAJIDNISAR/astra-lang.org.git` into a temp dir, then copied to `<local-disk>/aqelith-lab/astra@ca24fb9/` (4.3 MB). Result: OK. Files: 11 HTML pages, `assets/status.json`, `assets/playground/{astra.wasm,examples.json,playground.js,worker.js}`, `tools/sync-status.mjs`, `.github/workflows/static.yml`. No Rust source, no tests, no benchmark scripts.
5. `shasum -a 256 assets/playground/astra.wasm` and `file` -> hash above, "WebAssembly (wasm) binary module version 0x1 (MVP)".
6. Separate fetch step (network allowed): `docker pull node:22-alpine` -> OK (`node --version` = v22.23.4).
7. Sandboxed execution (no network, source mounted read-only; the driver script lives outside the downloaded directory):
   `docker run --rm --network=none -v "<lab>/astra@ca24fb9/assets/playground:/w:ro" -v <scratch>/drv:/d:ro node:22-alpine node /d/t.mjs`
   and the same with `/d/t2.mjs`. The driver instantiates the wasm with the same `pg_alloc/pg_call/pg_free/pg_result_len` protocol as `worker.js:35-43` and sends JSON requests `{op, source, stdin}`. Container exited 0; no crashes, no traps. Build time: not applicable (prebuilt wasm, no compile step). Native `astra build`, `mifa`, `astra mcp`, `scripts/validate_metrics.sh`, `benchmarks/run.py`, and `cargo test` could NOT be run: none are public.

Selected verbatim observations from step 7 (JSON trimmed to the relevant fields):

| Request (op) | Observed response |
|---|---|
| check `let total = 5` / `print(totl)` | `E0002` line 2 col 7, message `Undefined variable 'totl'; did you mean 'total'?`, `"fix": null` |
| fix, same source | `"applied": 0`, source unchanged, error remains |
| fix `fn main() -> int:` with `let total = 1` / `print(totl)` | `"applied": 1`, source rewritten to `print(total)`, `success: true` |
| check `let n: int = "hello"` | `E0004`, `line: null`, `column: null` |
| check `add(1)` for 2-arg fn | `E0005`, `line: null`, `column: null` |
| check enum match missing arm | `E0021` line 5 col 5, `"fix": null` |
| check `fn (:` | `E0001` line 2 col 1 `expected an indented block` |
| fix `ad(1, 2)` (typo of `add`) | `E0003`, `applied: 0`, `fix: null`, no "did you mean" |
| run `fib(n - 1) + fib(n - 2)`, `fib(20)`, `print(true)` | `{"exit":0,"stdout":"6765\ntrue\n"}` |
| run `fib(n-1) + fib(n-2)` (no spaces) | `E0001 Expected ')' after function arguments` line 4 col 17 (parser rejects `n-1`) |
| run `9223372036854775807 + 1` | `R0001 integer overflow in addition` |
| run `1 / 0` | `R0002 integer division by zero` |
| run `[1,2][5]` | `R0004 index out of bounds: the length is 2 but the index is 5` |
| run unbounded recursion | `R0005 stack overflow (recursion too deep: 400 nested calls)` |
| fmt braces-style source | rewritten to indentation style (canonical) |
| fmt on already-canonical source | identical output (idempotent on this one sample) |
| ir `print(1 + 2)` | textual LLVM IR with `llvm.sadd.with.overflow.i64`, `@astra_trap`, `@astra_rc_release`, `@puts` |
| check `@pure fn` calling `read_line()` | `E0014`, phase `capability`: `` `f` is marked @pure but uses `io`: f -> read_line `` |
| same input checked twice | byte-identical JSON |
| `op: "bogus"` | `{"error":"unknown op 'bogus'","ok":false}` |
| check with no `source` field | `success: true` (silently accepted) |

## Feature audit

Evidence column: `wasm` = observed first-hand against `astra.wasm` (interpreter/front end only); `doc` = project's own page, a claim I could not confirm against source; paths are in the website repo unless a URL is given.

| Feature | Classification | Evidence |
|---|---|---|
| Public compiler source | NV (source private until "v1.0-beta") | https://astra-lang.org/about.html; github.com/MAJIDNISAR/Astra 404 |
| Public native release binaries | Not verifiable (private tester access only) | https://astra-lang.org/download.html |
| JSON diagnostics (shape: phase, kind, error_code, message, file, line, column, severity, suggestion, fix) | Implemented but experimental (shape observed in wasm; CLI flag `--error-format=json` not runnable) | wasm outputs above; doc https://astra-lang.org/agents.html |
| Stable error codes (E0001-E0022, R0001-R0007, E0099) | Partially implemented (codes exist and appear; "never renumbered" promise is a doc claim; E0015/E0019 absent from catalogue; v0.x alpha) | wasm; agents.html code table |
| Line/column on every diagnostic | Partially implemented (E0004, E0005 returned null line/column in wasm; docs admit signatures/imported modules have none) | wasm; agents.html "Typecheck errors" paragraph |
| Machine-applicable fixes | Partially implemented (works for some E0002 cases; E0003, E0021, E0022 had no fix; docs list only E0002, E0003, E0022) | wasm "fix" ops above; progress.html "Machine-applicable fixes" row |
| `astra fix` apply-and-reverify loop | Implemented but experimental (op `fix` applied 1 edit and returned `success:true`; inconsistent: top-level `let`/`print` script got `applied:0` despite "did you mean") | wasm |
| Deterministic repair (same input -> same output) | Implemented but experimental (identical output on repeated check; one sample) | wasm "determinism" ops |
| Interpreter (`astra run`) | Implemented but experimental (runs fib, traps overflow/div0/OOB/recursion; 400-call depth limit in wasm vs 250,000 claimed native) | wasm; playground.html |
| Interpreter/native parity | Not verifiable (claimed: parity tests + differential fuzzer + 20 showcase + 8 real-world examples; native side not runnable) | progress.html "Interpreter with native parity"; benchmarks.html "Correctness metrics" |
| Known parity gaps | Marketing claim contradicted by own docs in part: recursion limits differ (400 vs 250,000); `await` not in interpreter; WASM archive runs single-file programs only | playground.html; progress.html "Not implemented yet" row |
| LLVM IR backend | Implemented but experimental (IR emitted by wasm `ir` op is real LLVM textual IR with overflow intrinsics; native codegen/link pipeline not runnable) | wasm `ir` op; progress.html "Native compilation" |
| Native compile pipeline (IR -> opt -> llc -> link) | Not verifiable (external `llc` + clang/Xcode CLT required per download.html) | download.html |
| Legacy backends consolidated | Not verifiable (claimed removed 2026-11-01; "ir.rs is the only code generator") | progress.html work queue item 7 |
| Effect restrictions (`@pure`, inferred effects, `--deny fs,env`) | Partially implemented (`@pure` + `read_line` -> E0014 observed in wasm; `--deny` CLI and `[policy] deny` untestable) | wasm capability test; index.html "--deny fs,env" |
| Capability model (declared capabilities, sandboxed tools) | Planned (only "capabilities" = effect inference + deny list exist; `tool`/`agent` keywords and capability-scoped sandbox are listed as Planned) | assets/status.json "planned"; vision.html |
| Runtime memory (ARC scope-based reclamation) | Partially implemented (docs admit leaks in if/match branches, call-argument locals, stored/reassigned locals, `?`/throw exits, struct str fields; IR shows `astra_rc_release`) | progress.html ARC row; wasm `ir` op |
| Memory numbers (231 MB -> 1.3 MB peak etc.) | Not verifiable (measured by author 2026-11-03; harness private) | assets/status.json "memory" |
| Formatter (`astra fmt`) and canonical syntax (indentation) | Implemented but experimental (brace -> indent rewrite observed; idempotent on one sample; "proves the program did not change" is unverified) | wasm `fmt` op; index.html |
| Package manager Mifa (init/add --path/lock/build/run/test) | Not verifiable (binary private); registry: Planned (`mifa search` says registry not deployed; `publish --dry-run` only) | mifa.html; progress.html |
| Deterministic lockfile (sha256-identical) | Not verifiable (author's script only) | benchmarks.html "Deterministic dependency resolution" |
| Signed/checksummed registry packages | Planned | vision.html |
| MCP server (`astra mcp`: verify, fix, run, build, format, explain, test) | Not verifiable (shipped in v0.13 per docs, hours old; binary private) | agents.html |
| Cross-platform builds (Linux x64, macOS arm64/x64, Windows x64) | Not verifiable (claimed 55/55 black-box checks per platform; Windows native linking first fixed in v0.12.1, 2026-11-05) | download.html; progress.html "Release testing" |
| WebAssembly build (front end + interpreter) | Implemented but experimental (observed working under Node 22 in Docker) | wasm run |
| Compile-to-WASM target (`--target=wasm32-wasi`) | Partially implemented (emits an object only; no WASI runtime build) | download.html; progress.html "WASM target" |
| Test suite "777 passed, 0 ignored" | Not verifiable (tests private) | progress.html; benchmarks.html |
| Benchmarks vs C/Rust/Go/Java/Node/Python | Not verifiable (author's harness, one Apple M2 Max, noisy machine, load avg 4-12) | benchmarks.html |
| Verify latency "4-5 ms" | Not verifiable (CLI timing; docs elsewhere say "~100 ms" on agents.html for verify; benchmark real-program table says ~7 ms) | agents.html vs benchmarks.html |
| Language features (structs, enums+match, Result+?, tuples, closures, generics w/ static dispatch, nominal traits) | Partially implemented (examples.json has 21 runnable programs; many mainstream features missing: Map/Set, generic structs, `elif`, `sqrt`, list pop/sort, closures as values) | assets/playground/examples.json; progress.html "Not implemented yet" |
| Python interop | Planned | status.json |
| `tool`/`agent`/`prompt`/`memory` keywords | Planned | vision.html; status.json |
| GPU (NVPTX) target | Planned (v1.0) | progress.html |
| LSP | Planned ("LSP alpha" in Next) | status.json |
| Hosted registry (mifa.dev) | Planned | mifa.html; vision.html |
| License MIT | Marketing claim only for the compiler (website repo has no LICENSE file; compiler source unavailable; JSON-LD and footer say MIT) | index.html JSON-LD; repo `git ls-files` |

## Architecture notes

What is documented (not independently confirmed beyond the wasm):

- Rust implementation (`cargo test`, `clippy -D warnings`, `cargo audit`, `astra::compiler::verify(source)` / `compile_to_result(source)` shown on agents.html). Source layout hints: `ir.rs` is the single code generator; `tests/regression_tests.rs`, `tests/showcase_examples_tests.rs`, `tests/ir_validity_tests.rs`, `scripts/validate_metrics.sh`, `benchmarks/run.py`. None are visible.
- Pipeline: lexer/parser (indentation + still accepts braces) -> import resolution (before typecheck) -> closure lift -> ad-hoc type inference + check (full Hindley-Milner "still unwired" per progress.html) -> capability/effect pass (phase `capability`, observed) -> LLVM IR -> `opt` -> `llc` -> system linker. A C runtime is embedded in the compiler binary (v0.11.1).
- Two execution engines: LLVM native and a tree-walking interpreter (slot-resolved) used for `astra run`, the playground and the wasm build. Parity is enforced by tests and a program fuzzer per the docs.
- Runtime traps (overflow via `llvm.sadd.with.overflow`, div0, shift range, bounds, recursion depth) are in the emitted IR (`@astra_trap`) and in the interpreter.
- Memory: reference-counted heap values with scope-based release (`astra_rc_release` observed in IR), single-owner optimisation, retain-on-alias. No cycle collector is mentioned; leaks are documented as "bounded and safe, never a double-free" (doc claim).
- Agent interface: one JSON object per invocation; fields "only grow"; `astra explain E####`; MCP stdio server whose tools return the same JSON as the CLI; programs run in a separate process with a time limit and capped output, and paths outside the start folder are refused (doc claim).
- WASM module protocol (observed in `worker.js`): a tiny ABI (`pg_alloc/pg_call/pg_free/pg_result_len`) taking a JSON request `{op: run|check|fix|fmt|ir, source, stdin}` and returning JSON; program stdout streams through `env.pg_out`. This is a clean, embeddable "compiler as JSON function" design.
- Site tooling: `tools/sync-status.mjs` regenerates page fragments from `assets/status.json` so the roadmap, feature globe and tables share one source of truth.

## Security considerations

- Source and native binaries are closed, unsigned ("The binaries are unsigned", download.html) and distributed privately; supply-chain trust cannot be assessed. I did not run any native binary.
- The wasm module executed in a Node sandbox with `--network=none`; the module has exactly one import (`pg_out`), so it has no file, env, or network access by construction. Program-level file/env/args are described as unavailable in the playground.
- A `read_file` call returned a `Result<str,str>`; printing it was rejected by the type checker, so I did not confirm file-read behaviour at runtime in wasm.
- "Capabilities" = inferred effects (`io`, `fs`, `env`) that `@pure` and `--deny` can forbid at compile time (E0014). This is a static, opt-out restriction on the compiler's built-in calls. There is no runtime enforcement, no per-dependency capability, and no manifest-declared grant that third-party packages must satisfy. The stronger "tool is a typed sandboxed capability ... executes with declared capabilities only" is vision text (Planned).
- MCP server: claims path confinement and process isolation with time/output limits; unverified. A single-developer language server exposing `run` and `build` to agents is a high-value attack surface.
- Registry plan promises checksummed, signed packages; today only path dependencies work and `publish --dry-run` checksums locally.
- No SECURITY.md, CVE history, or audit is public. Only `cargo audit` in CI is claimed.
- The compiler "never panics on malformed input" is a claim backed by a fuzzer per docs; I saw no panics or traps across malformed/odd requests in the wasm (`fn (:` and an unknown op both returned structured errors; a request with no `source` returned `success:true`, which is lenient rather than strict).

## Performance evidence

All numbers are the author's, from benchmarks.html (Apple M2 Max, macOS 26.1, Python 3.14.4 per page text; Python 3.11 in the table caption; machine not idle). I reproduced none of them and the harness is private. The page itself records a correction: an earlier version claimed C/Rust/Go-class speed and was wrong because the timing harness spawned Python per measurement.

Median ms as published (Astra | Rust -O | C -O2 | Go | Java | Node | Python | Astra interpreter):

- fib(38): 144 | 113 | 112 | 128 | 140 | 360 | 3,558 | 4,673
- 200M-iteration loop: 113 | 45 | 46 | 106 | 165 | 1,219 | 22,554 | 7,514
- Collatz 1..1M: 177 | 129 | 124 | 151 | 234 | 696 | 7,092 | 7,571
- Mandelbrot 1000x1000: 56 | 68 | 66 | 58 | 103 | 130 | 6,102 | 5,755
- 10M array + 10 passes: 100 | 68 | 69 | 123 | 312 | 274 | 9,282 | 5,533
- 50M small structs: 45 | 32 | 38 | 48 | 82 | 121 | 8,313 | 11,402
- 3M string concats: 123 | 230 | 214 | 86 | 98 | 109 | 589 | 703

Other published claims: verify 4-5 ms (agents.html also says ~100 ms for the same command; 7 ms in the real-programs table); native compile 68 ms for a 2M-iteration loop; 8/8 real programs match Python, median 7.2x faster native, but 1.78x more lines than Python; memory peak 231 MB -> 1.3 MB for call-in-loop cases (status.json). The table's loop rows are within 2.5x of C, which is plausible for LLVM -O2 with overflow checks; the interpreter being near CPython speed is plausible. Rust "checked" column exists for fairness. Caveats: tiny kernels, single machine, cold-start and macOS scan effects acknowledged by the author, Rust numbers rank above Astra on 5 of 7 rows.

First-hand: the only performance observation I made is that `astra.wasm` loads and answers all ~25 requests in one short Node process (not timed; no per-op timings recorded).

## Known limitations

From the project's own pages unless marked (obs) = observed by me:

- Source private until v1.0-beta; binaries only for testers; no public issue tracker, no public releases, no external contributors visible (1 star on the website repo).
- Alpha, version churn of 4 releases in ~2 days; contract stability promised ("codes never renumbered") but only 21 codes exist and the surface is moving.
- Fixes cover few cases; E0021/E0020 fixes are "next". (obs) fix is not reliable even for E0002 in a top-level script, and E0003 gave no suggestion.
- (obs) null line/column for type mismatch and arity errors.
- (obs) Parser rejects `n-1` without spaces (`Expected ')' after function arguments`), a basic lexing gap; same program with `n - 1` works.
- Missing: Map/Set, generic structs, `@derive`, trait default methods, list `pop`/sort/slice, `elif`, `sqrt`/math, closures as values, multi-field enum variants, `await` in interpreter, real async runtime (async lowers synchronously).
- try/catch is function-local only (no unwinder); `throw` in a callee aborts.
- Equality `==` only on int/float/bool/str.
- ARC leaks remain in listed cases; no cycle story documented.
- Interpreter recursion limit 400 in playground vs 250,000 native.
- Native build needs external LLVM (`llc`) and clang/Xcode CLT; Windows needs the full LLVM archive; WASM archive cannot build native executables; `--target=wasm32-wasi` produces an object with no WASI runtime to link it.
- No LSP, no stdlib core, no hosted registry, no Python interop, no agent keywords, no GPU.
- Benchmarks cover 7 kernels; 5 standard Benchmarks Game programs are omitted because the language lacks features.

## Ideas worth adopting for AQELITH

1. Compiler as a pure JSON function. The wasm ABI (`{op, source, stdin}` -> JSON) is a minimal, sandboxable, embeddable agent interface; an AQELITH wasm module with one import (output sink) gives zero ambient authority by construction.
2. One diagnostic object per invocation with `phase, kind, error_code, line, column, severity, suggestion, fix{edits}`; edits as half-open line/column ranges. Adopt, but make line/column mandatory (Astra returns null for several codes) and make fixes content-hashed against the source version so stale edits are rejected.
3. Categorical, permanent error codes with a "typical agent repair" column and `explain <code>` command; add machine-readable severity/fixability class per code so frameworks route without parsing text.
4. Separate runtime codes (R0001+) from compile codes, emitted as JSON on stderr while stdout stays the program's.
5. Effect inference + `@pure` + deny policy (E0014 with the call chain `f -> read_line`). Taking it further (declared capability set in the manifest, enforced for dependencies and at runtime) is the gap Astra leaves open.
6. `fmt` rewrites to a single canonical syntax and should "prove program unchanged" (AST equality before/after); make that a hard, tested invariant with idempotence property tests, and refuse non-canonical input in agent mode.
7. Status as data: `assets/status.json` + a sync script generating the roadmap, with classes works/shipped/next/planned. AQELITH can publish an evidence-linked feature matrix generated from CI (each row links a passing test).
8. Honest-benchmark discipline: output-verified harness, in-repo correction note, per-workload comparison to checked and unchecked Rust. Adopt the discipline and avoid the self-inflicted harness bug by timing in-process.
9. Differential interpreter-vs-native fuzzing as the parity gate, from day one.
10. Counter-lessons: do not publish a roadmap page as proof while source is closed; do not ship an MCP server and a fix engine in the same release as basic lexing gaps (`n-1`); do not label an effect lint a "capability model".

## Confidence

- Sources / identity: high. Project, author, status, and the private-source statement are stated on multiple pages and consistent with the 404s.
- Inspected version: medium. Commit and wasm hash are exact; the link between wasm and any source commit is unproven and the compiler version in the wasm is not self-reported.
- Commands executed: high for the items I ran (all re-runnable from the listed commands; outputs quoted from the run); the driver used only protocol facts from `worker.js`.
- Feature audit: medium overall. High for features I exercised in wasm (diagnostics, basic fix, fmt, IR, traps, @pure); low for everything native, MCP, Mifa, parity, cross-platform, which are Not verifiable.
- Architecture notes: medium-low. The pipeline is assembled from docs plus observed IR and wasm exports; no source read.
- Security considerations: medium-low. Static facts are solid; runtime posture of native binaries, MCP, and the registry is unverified.
- Performance evidence: low. All figures are the author's, unreproduced, single machine, and the page admits one earlier erroneous table.
- Known limitations: medium-high. Mostly the project's own admissions plus three first-hand findings (fix inconsistency, null positions, `n-1` parse failure), each reproducible with one request to the wasm.
- Ideas for AQELITH: medium. Judgment calls based on the above; items 1-4 rest on observed behaviour, items 6-9 on documented claims.
