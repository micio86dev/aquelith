# Competitor audit: AICore (keaz/aicore)

Auditor: research engineer, project AQELITH. Evidence = source/docs/tests/issue tracker at the pinned commit. The first pass could not build (Docker crash, see Commands); a second pass built the compiler and ran its test suite (Commands, item 8). Behavioural claims below are "by reading" unless they cite that run; the `aic` CLI, the examples and the benchmarks were not executed.

## Sources

- Repo: https://github.com/keaz/aicore (Apache-2.0, created 2026-02-19, 2 stars, 0 forks, 1 contributor `keaz`, ~359 commits, 0 tags, 0 GitHub releases at inspection)
- Issues: https://github.com/keaz/aicore/issues (82 open, 416 closed; numbers 217..498). Roadmap epic https://github.com/keaz/aicore/issues/420
- Key epics/issues cited: #421 (CORR), #424 (MEM), #426 (VERIFY), #447 (self-host scope), #476 (no release), #488 (FFI effect hole), #489 (SMT), #492/#493/#495/#497 (effect polymorphism, user effects, typestate, capabilities-as-values), #470-#474 (memory)
- Local clone: `<local-disk>/aqelith-lab/aicore@8976f47`
- In-repo docs read: `README.md`, `docs/architecture.md`, `docs/ir-schema.md`, `docs/effect-system.md`, `docs/reference/effects.md`, `docs/contracts.md`, `docs/type-system.md`, `docs/reference/memory.md`, `docs/spec.md`, `docs/async-event-loop.md`, `docs/security-threat-model.md`, `docs/agent-tooling/protocol-v1.md`, `docs/prod-readiness/language-production-roadmap.md`, `docs/selfhost-driver.md`, `docs/perf-baseline.json`, `docs/perf-budget.json`
- Source read (partial, targeted): `src/typecheck.rs`, `src/effects.rs`, `src/contracts.rs`, `src/ir.rs`, `src/ir_builder.rs`, `src/codegen/{mod,generator_core,divisor_facts}.rs`, `src/codegen/runtime/part01.c`, `src/patch_protocol.rs`, `src/daemon.rs`, `src/sandbox.rs`, `Makefile`, `.github/workflows/ci.yml`

## Inspected version

- Commit `8976f47754f835002ab1db3c7027b718aab07c33` ("Keep the working tree clean after CI (#446)"), committed 2026-10-11T13:02:46+05:30, branch `main` HEAD at clone time.
- Crate version `0.1.0` (`Cargo.toml:3`); no tag, no release (issue #476 states this and that docs promise an LTS branch `release/0.1`).
- Inspection date: 2026-11-05.
- Clone was `--depth 50`; commit total comes from the GitHub API pagination (`rel="last"` page 359 at per_page=1).
- Important timing fact: the maintainer's self-audit (issues #420-#498, opened 2026-10-10) targets commit `2a11587`. HEAD (`8976f47`, 2026-10-11) already contains fix commits for #429-#445 (e.g. `2447b92` "Guard integer division and remainder (#434)", `e4b6d56` "Give Vec value semantics (#438)", `5d42eb6` "Enforce contracts on impl methods (#433)"). Those issues were still OPEN on GitHub at inspection, so issue state lags code. I classify from HEAD code/commit evidence, not from issue state, and mark such items "not re-verified by execution".

## Commands executed

All on macOS host (Darwin 25.5.0) unless noted. Cloned code was never run on the host.

1. `git clone --depth 50 https://github.com/keaz/aicore tmp-aicore` then renamed to `<local-disk>/aqelith-lab/aicore@8976f47`. Result: OK, 903 files. `git tag | wc -l` = 0.
2. `gh release list -R keaz/aicore` -> empty. `gh issue list --state open` -> 82 items; closed -> 416.
3. Static reads only: `grep`, `wc`, `sed` over the clone. `.rs` total ~186.6k lines incl. tests (`wc -l`); `tests/unit_tests.rs` 11,101 lines; `src/typecheck.rs` 13,819; AIC-language self-host compiler `compiler/aic/libs/*/src/main.aic` 40,808 lines; `std/*.aic` 26,435 lines; C runtime `src/codegen/runtime/part0{1..5}.c` 30,779 lines. `grep -rn '#\[test\]' tests src | wc -l` = 1386. `find examples -name '*.aic' | wc -l` = 311.
4. Docker image prep (network allowed, separate step): `docker run --name aic-prep rust:1-slim sh -c 'apt-get update && apt-get install ... clang libssl-dev pkg-config make'` then `docker commit aic-prep aic-lab:rust`. Result: OK, `Debian clang version 19.1.7`. (`docker build` with a Dockerfile failed twice with `ERROR: failed to build: EOF`, buildx and legacy; the Docker daemon socket was gone for a while; workaround was run+commit.)
5. Dependency fetch (network allowed): `docker run --rm -v <clone>:/src:ro -v /private/tmp/aic-cargo:/cargo -e CARGO_HOME=/cargo aic-lab:rust sh -c 'cd /src && cargo fetch --locked'`. Result: OK in 14 s, 185 MB crates. Toolchain in image: `cargo 1.99.1`, `rustc 1.99.1 (2026-10-13)`.
6. Build (offline): `docker run --rm --network=none -m 3500m -v <clone>:/src:ro -v /private/tmp/aic-cargo:/cargo -v /private/tmp/aic-target:/target -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target aic-lab:rust timeout 1200 cargo build --release --locked --offline --manifest-path /src/Cargo.toml -j2`. First attempt had a typo (missing image name) -> `exit=125`, no work done. Second attempt: all 40-odd dependencies compiled, then at `Compiling aicore v0.1.0 (/src)` the Docker daemon died: `level=error msg="error waiting for container: unexpected EOF"`, `exit=125`, wall ~2 min. Afterwards `docker info` could not reach `~/.docker/run/docker.sock`. Cause not established (suspected memory pressure of the shared ~4 GiB Docker VM, which also hosts other unrelated containers). I did not retry: a second crash risks killing the user's other running containers.
7. First pass: not run (consequence of 6). Superseded for build and tests by item 8.
8. **Second pass (2026-11-05, after Docker was moved to the SSD and restarted).** On a copy of the clone in `aqelith-lab/build/`, `--memory=3g`, `CARGO_BUILD_JOBS=2`, toolchain `rustc 1.99.1` from the image (no `rust-toolchain` file in the repo). The image was extended with `docker run` + `docker commit` (`clang`, `libssl-dev`, `pkg-config`, then `python3`, `git`, `lld-19` for `wasm-ld`).
    - `cargo build --release --offline --locked`: exit 0, 119 s. The earlier crash did not recur with the 3 GiB limit and 2 jobs.
    - First `cargo test` run stopped at the first failing binary. Failures at that point were environmental: `wasm-ld-19` missing (2 wasm build tests) and `python3`/`git` missing (45 `selfhost_parity_tests`). I fixed the image, not the project.
    - Full run with those tools, `--network=none`, `--no-fail-fast`: 32 suites, **1342 passed, 5 failed, 2 ignored**, 473 s.
    - Triage of the 5 failures. Each was rerun alone with `--network=none`, in an image where `procps` had been installed in a separate fetch-only step (no project code ran with network on). An earlier triage that reran some of these with the network enabled was discarded because it broke the isolation policy; no result from it is used here.
        - `exec_signal_wait_for_sigterm_is_deterministic`: **passes** once `kill` (`procps`) is installed. Environment. (In the full run it failed with `send SIGTERM: ... NotFound`.)
        - `exec_fs_permission_error_maps_to_permission_denied`: expected permission-denied (2), got success (0), reproduced. The container user is uid 0 (`id -u` printed `0`). Root bypassing file permissions is the likely cause; I did not rerun as a non-root user, so this stays an inference.
        - `exec_net_udp_and_dns_helpers`: fails (expected 42, got 0) under `--network=none`. The test name suggests it needs network access, but I did not confirm that, because verifying it would require running project code with the network on, which the policy forbids without explicit user sign-off. Status: not explained.
        - `exec_runtime_tls_async_lifecycle_sustained_churn_is_leak_free`: fails with `free(): invalid pointer` (exit 1 instead of 0), reproduced alone. Cause not determined: it may be a defect in the project's C runtime, or an interaction with this image's OpenSSL (Debian, libssl 3.x). Not attributed.
        - `aic_selfhost_driver_tool_handles_supported_and_negative_commands`: "selfhost tool did not check its own source graph", reproduced alone (263 s). Cause not determined.
    - Not run: the `aic` CLI on examples, benchmarks, and any measurement of leak behavior outside the test suite.

## Feature audit

Classification legend: IT = Implemented and tested; IE = Implemented but experimental; PI = Partially implemented; PL = Planned; MC = Marketing claim only; NV = Not verifiable. "Tested" means tests exist in the repo and CI target names them; I did not run them. `path:line` are in the clone.

| Feature | Class | Evidence |
|---|---|---|
| Pipeline AST -> IR -> resolve -> typecheck -> static contracts -> LLVM text -> clang | IT | `docs/architecture.md:22-60`; `src/ir_builder.rs:6` (`build(&ast::Program) -> ir::Program`); `Makefile` targets `test-unit/golden/exec` |
| "IR is canonical source of truth" | PI | IR is always derived from source text (`ir_builder.rs:6`). `ir::Program` derives `Deserialize` but the only reader found is `ir-migrate` (`src/main.rs:2089`); no CLI compiles IR JSON back. Patch protocol edits AST/byte offsets in source (`src/patch_protocol.rs:13,582`). IR is canonical for emission/printing, not for authoring |
| Stable IR IDs (`SymbolId/TypeId/NodeId`), schema_version=1, interned types | IT | `src/ir.rs:7`, `docs/ir-schema.md`, `docs/id-allocation.md`; determinism tests `tests/e7_build_hermetic_tests.rs:478` |
| Versioned IR JSON + migration (`aic ir-migrate`) | IE | `docs/ir-schema.md:36-38`; `src/main.rs:2089` |
| Deterministic formatter, comment-preserving, re-parse self-check | IT | `docs/architecture.md:53`; `tests/formatter_tests.rs`, `tests/golden_tests.rs`; last commit `70ed120` "Keep comments and typed holes in aic fmt" |
| Structured diagnostics JSON with spans, line/col, fixes, SARIF, `aic explain` | IT | `README.md:767-815`; `docs/diagnostics.schema.json`; `src/sarif.rs`; line/col added in commit `0c22143` (#432). Per-code `explain` text incomplete (#467 open) |
| Ownership: single-owner moves | PI | Use-after-move E1277 only for `let b = a` at audit time (#471 text); `moved_by_binding: BTreeMap<String,Span>` (`src/typecheck.rs:302`); E1277 `typecheck.rs:4276`, E1278 `:4351`. Strings copy while structs move, undocumented (#471) |
| Borrowing: `&`/`&mut`, aliasing XOR mutation | PI | Lexically scoped, local-name-only targets, no lifetimes (`docs/reference/memory.md:46-58`); `BorrowState` `typecheck.rs:299`; assign-while-borrowed E1265 `:3746`. No deref operator, `RefMut` cannot write, returning `&local` passes `check` (#472, #445). Not Rust-grade |
| Lifetimes / dangling-reference prevention | PL | `memory.md`: "explicit lifetime syntax is not part of the surface language"; #472 open |
| Heap reclamation (String/Vec/struct free) | PL | Scope exit emits only `llvm.lifetime.end` (#470, `generator_core.rs` ~2874-2903 per issue). `--check-leaks` showed one leak per `int_to_string` (#470). `memory.md:33` "Buffers are never ... freed" |
| RAII for selected resources (`FileHandle`, `Map`, `Set`, channel, mutex, TcpReader) and `Drop` trait | IE | `docs/reference/memory.md:59-72`, `docs/type-system.md:227-229`; `generator_core.rs:1248,1616` (`DropScope`, `DropSlot`). Partial moves, join-point moves, unwind not done (#471, #474) |
| `Box[T]`/recursive data types | PL | #473; #445 says recursive types overflow rustc stack |
| Effect system: fixed taxonomy `io,fs,net,time,rand,env,proc,concurrency`, pure-by-default | IT | `docs/effect-system.md:21-40`; `src/effects.rs` (349 lines); E2001 `typecheck.rs:2105`; tests `tests/unit_tests.rs`, example `examples/effects_reject.aic` |
| Transitive effect closure with call-path diagnostic | IT | `docs/effect-system.md:32-33`; E2005 `typecheck.rs:2870` |
| Capabilities clause (`capabilities { io }`) | IE | Same token set as effects, so a second declaration of the same thing; E2007-E2009 (`typecheck.rs:2928,2945`). Maintainer's own roadmap: "capabilities duplicate effects" (`language-production-roadmap.md`); object-capability values are #497 (planned) |
| Effect polymorphism, user-declared effects, `panic`/`diverge` effects | PL | #492, #493, #487 |
| FFI effect tracking (`extern "C"`) | MC | `extern` cannot declare effects (E1066) and a pure fn can call one in `unsafe {}` (#488 open); `intrinsic` is guarded (E2102, `typecheck.rs:5201`) |
| Resource-protocol typestate (use-after-close, double-close) for built-in handles | IE | `docs/effect-system.md:41-63`; E2006 `typecheck.rs:3575`. Hard-coded list, conservative on branches; user-definable typestate is #495 (planned) |
| Design by Contract: `requires`/`ensures`/struct `invariant` parse+typecheck | IT | `docs/contracts.md`; `src/contracts.rs`; examples `examples/verify/range_proofs.aic`; contract purity enforced (effect-system.md:34) |
| Contracts lowered to runtime asserts (`aic_rt_panic`) | IT | `docs/contracts.md:20-24`; `contracts.rs:207` (`lower_runtime_asserts`). Impl-method contracts were silently ignored (#433), fixed in `5d42eb6` (not run-verified) |
| Static contract proving | PI | Interval arithmetic over `+ - neg` and comparisons; does not assume `requires` when proving `ensures`; never checks `requires` at call sites (#489, open). Result codes E4001-E4005 |
| SMT-backed verification, `old()`, quantifiers, loop invariants, refinement types, trait-method contracts | PL | #489, #490, #494, #491 |
| Contract-driven optimization: `requires b != 0` elides div guard | IE | `src/codegen/divisor_facts.rs:1-12` (literal or param proven non-zero by `requires`, never reassigned). Commit `2447b92` (#434). Not run-verified |
| Async: `async fn`/`await`, `Async[T]` | IE | Await lowered to poll frame + `aic_rt_async_drive` (`generator_core.rs:1635,3593-3620`; `runtime/part03.c:4257`). Docs say single-process reactor, 1..32 worker threads default 1 (`docs/async-event-loop.md:7-13`). Windows server paths not covered (#391) |
| Async net/TLS/fs submit-wait-cancel-poll API | IE | `docs/async-event-loop.md:15-80`; tests named in the doc matrix (`exec_net_async_*`); not run |
| Structured-concurrency guarantees / Send-Sync-like data-race checking | NV | Not found in typecheck; concurrency payloads are JSON-based (#484 "Native generic concurrency payloads (no JSON)") |
| Generics, traits, `dyn Trait`, coherence | PI | Works for basic cases; no default methods (#477), assoc types (#478), supertraits (#479), blanket impls (#480), `derive` (#482), generic Map keys (#483) |
| Exhaustiveness/reachability for nested patterns; check/build parity | IE | Fixed at HEAD: `effb699` (#442), `eb82979` (#443/#444), `de75e3c` (#445 "Make every program that passes aic check also build"). Not run-verified; issues still open |
| Integer overflow semantics | PL | Wraps silently; checked arithmetic is #486 |
| Division/modulo by zero safe | IE | Guards + `MIN/-1` guard: `divisor_facts.rs:3-4`; commit `2447b92` |
| Native backend via LLVM IR text + clang; C runtime (~30.8k lines) | IT | `docs/architecture.md:55-60`; `src/codegen/*`, `src/codegen/runtime/*.c`; `make test-exec` |
| Self-hosted compiler in AIC (`compiler/aic`) | IE | 40.8k lines AIC; per #447 (audit time) passes `check` on 13/36 core examples vs Rust 35/36, references 24 of 446 runtime symbols, rejects `--release/--target/-O`. Rust compiler is default. #419 (retire Rust reference) open |
| Agent tooling: `check --json`, `contract --json` version negotiation, daemon JSON-RPC, `query`, `context`, `symbols`, `patch`, `testgen`, `checkpoint`, `session`, `validate-call/type`, `suggest-effects/contracts` | IE | `docs/agent-tooling/protocol-v1.md` (14 JSON schemas listed); `src/daemon.rs:183`, `src/patch_protocol.rs`, `src/context_query.rs`, `src/session.rs`, `src/checkpoint.rs`; tests `tests/agent_protocol_tests.rs`, `tests/agent_recipe_tests.rs`, `tests/e7_cli_tests.rs`. Quality gap: `aic testgen` emits placeholder `assert(true)` (#496) |
| LSP server, DAP, VS Code extension | IE | `src/lsp.rs` (6,672 lines); `tests/lsp_smoke_tests.rs`; extension publishing open (#217) |
| Determinism: stable IDs, sorted diagnostics, hermetic build manifest, reproducible builds | IT (compiler-side) | `README.md:753-761`; `tests/e7_build_hermetic_tests.rs:478`; `make repro-check` -> `scripts/ci/repro-build-check.sh`. Byte-identical native binaries across hosts: NV (needs clang version pinning; not run) |
| Runtime sandbox for `aic run` | IE | `src/sandbox.rs`: Linux-only `prlimit` wrapper, limits only (`:180,194`); not a capability/syscall sandbox |
| Supply chain: lockfile, checksums, SBOM, signed provenance (HMAC), pinned workflows | IE | `docs/security-threat-model.md`; `src/release_ops.rs`; `.github/workflows/security.yml`. Single shared HMAC key (not asymmetric signing) |
| Test harness: `aic test`, `#[test]`, `#[property]` with shrinking, fuzz, differential, conformance | IT | `src/test_harness.rs`, `src/property_test_runner.rs`; `tests/e8_*`; 1386 `#[test]` attrs |
| Performance gate | PI | `docs/perf-baseline.json` and `perf-budget.json` hold round hand-set numbers (parser 40 ms, typecheck 70, codegen 90; budgets 300/500/700 ms, tolerance 35%); gate in `src/perf_gate.rs`, `tests/e8_perf_tests.rs`. No runtime-speed benchmarks vs other languages found |
| Release maturity | MC | README "Status" table shows every row "Implemented"; maintainer's own audit (#475 "Make docs match the implementation", `language-production-roadmap.md`) says docs overstate; no tag, no binaries (#476); CI on `main` was red (#429, fixed in commits of 2026-10-11) |

## Architecture notes

- Two compilers coexist. The default is the Rust reference (`src/`, ~186k lines incl. tests). The AIC self-host (`compiler/aic/libs/{lexer,parser,ast,ir,frontend,semantics,typecheck,backend_llvm,driver}`) mirrors it and is held to parity by `tests/selfhost_parity_tests.rs` (4,881 lines). Docs in `docs/architecture.md` and `docs/reference/effects.md` point at `compiler/aic/.../main.aic` paths while the real, active logic is in `src/typecheck.rs`: docs path references are partially stale (global search-replace artifact).
- Stages (docs/architecture.md:22-60): load -> lex/parse (error recovery) -> `ir_builder::build` -> `effects::normalize_effect_declarations` (sort/dedup effect and capability sets) -> `resolver` -> `typecheck::check` (types, effects, contract purity, borrow, exhaustiveness, resource protocols in one 13.8k-line file) -> `contracts::verify_static` -> `contracts::lower_runtime_asserts` -> `codegen::emit_llvm_with_options` -> `clang`. Backend is textual LLVM IR emitted by Rust string pushes (`fctx.lines.push(format!(...))`), not an LLVM binding.
- IR design: AST-lowered, ID-stamped (deterministic traversal), types interned by canonical text `repr`, generic instantiations deduplicated and ordered by key, `Program.schema_version`. IR keeps `is_async`, `is_intrinsic`, `intrinsic_abi`, `Borrow{mutable}`, `Await`, `Try`. There is no separate typed/ownership MIR: borrow/move/effect checks run on the same tree-shaped IR in `typecheck.rs`; codegen then computes drop slots (`DropScope/DropSlot`). Contracts are IR-to-IR rewritten (`lower_runtime_asserts`) before codegen.
- Compiler/runtime boundary: `std/*.aic` declares `intrinsic fn` stubs (signature only); Rust codegen maps intrinsic names to C runtime symbols (`aic_rt_*`) and links the concatenated C runtime. Before `cb74ac7` 223 std wrappers had dummy bodies intercepted by name (#439); that commit replaces them with declarations (not run-verified). The runtime is ~30.8k lines of C with pthread/Windows shims (`part01.c:37,105-120`), so the trusted base is large and C.
- Async model: poll-frame state machine for `async fn`, driven by `aic_rt_async_drive`; I/O through a bounded-queue reactor with an optional worker pool (`docs/async-event-loop.md`). Not a general work-stealing executor; effect accounting is unchanged across `await` (`effect-system.md:25`).
- Value semantics quirk: `Vec[T]` is a value but copies share a buffer using a per-buffer "frontier" table so that in-place push is safe (`docs/reference/memory.md:17-33`, `runtime/part01.c:1953-1963`). `Map`/`Set`/`ByteBuffer`/handles are reference-semantics. That mix is why free() (MEM-T2) is hard and depends on #438.
- Determinism: IDs by traversal order, sorted effect sets, sorted diagnostics, hermetic build manifest and lockfile checksums. Evidence is in tests and docs; I did not measure bit-reproducibility of native artifacts.
- Process observation: the repo is largely agent-written ("written mainly using ChatGPT Codex", README line 8), single committer, ~359 commits in ~8 months, with a long tail of gated epics (E1..E9, "QV-T", "REST-T", "AG-T" IDs). The maintainer then audited it with ~200 probe programs and published an honest 70-item remediation plan; most Phase 0-1 items landed within one day (2026-10-11).

## Security considerations

- Threat model (`docs/security-threat-model.md`) covers compiler CLI, release provenance, CI. It explicitly accepts: HMAC-key compromise invalidates provenance, hard sandbox limits are Linux-only (`prlimit`), manual release bypass possible.
- Language-level safety gaps at audit time (some fixed at HEAD but unverified): silent integer wrap (#486 open), unguarded div/mod (fixed `2447b92`), Vec aliasing (fixed `e4b6d56`), closure capture write loss (now rejected, `cbbadf6`), unknown types passing `check` (fixed `831a127`). Still open: no memory reclamation (unbounded growth for services), `RefMut` unusable, dangling refs (`&local` return passes `check`, #445), `panic` = `exit(1)` with no unwind (#424, #474).
- Effects are a coarse, fixed 8-name set. Authority is static annotation, not an unforgeable value; `capabilities` repeat `effects`. `extern "C"` plus `unsafe` bypasses effect tracking (#488). Effects do not stop a dependency from doing anything its declared effect allows (no per-path/host scoping).
- Large C runtime (net, TLS via OpenSSL, crypto, regex, JSON, http server in C) compiled into every program: memory-safety bugs there bypass every language guarantee. 30.8k lines with ~114 `malloc` occurrences (`grep -c malloc` over the five parts).
- Compiling untrusted `.aic` runs clang; builds shell out. Sandbox is a ulimit wrapper only.
- `aic run` isolates nothing on macOS/Windows (`sandbox.rs:194` errors "use --sandbox none on this platform").
- Our handling: cloned code was never built or run on the host; the build ran in a `--network=none` container; in the first pass it crashed the Docker VM at the first compile of the crate (cause unknown, not attributed to the project; the second pass with a 3 GiB limit and 2 jobs built fine). Tests also ran only in the container.

## Performance evidence

- No performance measured by me. Release build took 119 s in a 4 GiB Docker VM with 2 jobs and the test suite 473 s; these are not compiler or runtime speed claims.
- In repo: `docs/perf-baseline.json` {parser 40.0 ms, typecheck 70.0, codegen 90.0, dataset `examples/e8/large_project_bench`} and `docs/perf-budget.json` (max 300/500/700 ms, 35% tolerance). The numbers are round and look hand-set; I did not find the harness output that produced them. Treat as budgets, not measurements.
- Maintainer-reported (issue text, unverified by me): `aic check` took 40-140 s on a one-line file due to project-directory walk (#431; fix commit `38d9875` "Resolve imports without walking the project directory"). Runtime: 20k string-loop iterations leaked ~60k allocations (#470). `Vec` ops: push amortized O(1) on latest value, `set`/mid-`insert` O(n) (`memory.md:25-33`).
- `docs/runtime-performance/` holds design notes (small-string optimization, tail-call optimization) with no comparative numbers found in the files I read.
- No benchmarks against Rust/Go/C. `benchmarks/service_baseline` exists (directory with 6 entries, not read in depth).

## Known limitations

From the maintainer's own tracker (open at inspection) and docs:

- Memory: no freeing of heap values, no Box, no recursive types, no deref/write-through refs, panic kills process (#469-#474).
- Verification shallow: interval-only static prover, no call-site precondition checks, no SMT, no `old()`/quantifiers/loop invariants, no refinement types, no user typestate (#489-#495).
- Effects: no polymorphism for higher-order functions, no user effects, `extern` hole, capabilities not values (#488, #492, #493, #497).
- Syntax gaps: no field/index/compound assignment, `if let`, struct-like enum variants, casts, indexing sugar, labeled break, block comments (#448-#463). Several of these are everyday constructs (T1..T16), so real programs are awkward to write.
- Traits: no default methods, assoc types, supertraits, derive, `Eq/Ord/Hash/Display` core traits (#477-#483).
- Integer arithmetic wraps silently (#486).
- Self-host compiler far from parity (#447); Rust compiler remains required.
- No release, no prebuilt binaries; CI red on main until 2026-10-11; Windows async server coverage missing (#391, #429, #476).
- Docs/README overstate: "Implemented" status table vs open issues (#475). Some doc path references stale.
- Single maintainer, 2 stars: bus factor 1, almost no external usage evidence.

## Ideas worth adopting for AQELITH

1. Agent protocol discipline: versioned `contract --json --accept-version` negotiation, one JSON Schema per response, a daemon JSON-RPC mirror of the CLI (`docs/agent-tooling/protocol-v1.md`). Cheap, high leverage for agent users.
2. Diagnostics shape: stable code + byte span + 1-based line/col + `help[]` + machine-applicable `suggested_fixes`, plus SARIF and `explain CODE`. Adopt the shape; commit to complete per-code explain text from day one (their #467 gap).
3. Deterministic ID scheme: IDs from deterministic traversal, interned types by canonical text, sorted generic instantiations, versioned IR with a migrator. Take the idea but make IR genuinely authorable/ingestible (their IR is output-only), so "IR-first" is true.
4. Contract-informed codegen: elide runtime guards when `requires` proves a divisor non-zero (`divisor_facts.rs`). Small, demonstrable win for "contracts are not just asserts".
5. Effect call-path provenance in errors (E2005 reports the path that introduces an effect) and "contracts must be pure" as a hard rule.
6. Patch/checkpoint/session commands (`patch` with byte-offset preconditions, `checkpoint` restore/diff, `validate-call`) as first-class compiler subcommands for agent edit loops; but base them on IR node IDs rather than text offsets.
7. Process lesson: publish the failure audit (probe-based, with repro programs) and gate on it. Their "check pass implies build pass" parity rule (#445) and "no placeholder success paths" repo rule are worth copying as CI invariants.
8. Cautionary design points to avoid: duplicating `capabilities` and `effects` (pick one, make capabilities values); fixed effect set without polymorphism; `Vec` shared-buffer value semantics (forces a frontier table and blocks freeing); lexical-only borrows without lifetimes (soundness holes: `&local` returns, no write-through); decide the memory model (ownership-drop vs RC) before the stdlib, not after (their MEM-T1 is item 41 of 70); shipping a README status table ahead of verified behaviour.

## Confidence

| Section | Level | Why |
|---|---|---|
| Sources / Inspected version | high | Commit, dates, release/tag counts read directly from git and `gh`; commit total from API pagination |
| Commands executed | high | Exact commands and failure text recorded; the first-pass Docker crash cause is unknown; second-pass results copied from the test logs; 3 of 5 test failures remain unexplained (UDP/DNS, TLS, selfhost) and 1 is an unverified inference (fs permission, root) |
| Feature audit: effects, contracts, diagnostics, agent protocol (existence) | medium | Read docs, key source anchors, and test names; nothing executed |
| Feature audit: ownership/borrow/memory | medium-high | Corroborated by three independent sources (code structs, docs `memory.md`, maintainer issues) |
| Feature audit: items fixed at HEAD (#430-#445) | low-medium | Commit titles and a few code anchors only; issues still open; no run-time confirmation |
| Async | medium | Codegen anchors and docs read; semantics of scheduling/cancellation not exercised |
| Architecture notes | medium | From docs plus targeted source reads; the 13.8k-line `typecheck.rs` and codegen read only in parts |
| Security considerations | medium | Threat model and sandbox code read; runtime C not audited line by line |
| Performance evidence | low | No execution; repo numbers look hand-set and have no producing harness located |
| Known limitations | high | Mostly taken from the maintainer's own tracker and roadmap, at audit commit `2a11587`; some may already be fixed at HEAD |
| Ideas | medium | Judgement, grounded in the evidence above |
