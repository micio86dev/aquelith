# Competitor audit: Zerolang (vercel-labs/zerolang)

Audit date: 2026-11-05. Auditor: AQELITH research engineer (automated agent). Language: English.

## Sources

- Repo: https://github.com/vercel-labs/zerolang (Apache-2.0, created 2026-05-15, ~5.4k stars at inspection)
- Site (docs/landing): https://zerolang.ai/ (fetched; no numbers for memory/speed/size/tokens)
- Site (landing): https://zerolang.app/ (fetched; footer says "Zerolang Community"; not verified as operated by Vercel; only number: "sub-10 KiB native binaries")
- Releases: https://github.com/vercel-labs/zerolang/releases (v0.1.3 ... v0.3.4, latest 2026-06-13)
- Issues: https://github.com/vercel-labs/zerolang/issues (54 open, 20 closed, #442 highest at inspection). Read in full: #348, #104, #318, #435, #181; titles of the first 40 open.
- Local clone (read-only reference): `<local-disk>/aqelith-lab/zerolang@7e1a64d2`
- Paths below are relative to that clone. Key files: `README.md`, `AGENTS.md`, `skill-data/*.md`, `docs/articles/concepts/*.md`, `native/zero-c/src/program_graph_*.c`, `CHANGELOG.md`.

## Inspected version

- Commit `7e1a64d27cc37671df31c6370890bce86f5135e1` (short `7e1a64d2`), "Add Labs status badges to README (#433)", dated 2026-09-16. Branch `main`, 1188 commits, 1187 by one author (Chris Tate); 1 by another.
- 16 commits after tag `v0.3.4` (latest release, 2026-06-13, binary self-reports `zero 0.3.4`). Last push 2026-09-16, i.e. ~7 weeks before inspection; last release ~5 months before.
- The built binary reports `zero 0.3.4 (build 7e1a64d2)`.
- Inspection date: 2026-11-05.

## Commands executed

All build/run steps ran in Docker (`node:22-bookworm`, digest `sha256:e273cf2a...e9c7`, linux/aarch64, gcc 12.2.0, node v22.23.4) with `--network=none`. The image pull was done by the Docker daemon beforehand. Nothing from the clone was executed on the host. Clone: `git clone https://github.com/vercel-labs/zerolang.git`, then `git checkout` not needed (HEAD = `7e1a64d2`).

Build attempts (all verbatim outcomes):

1. `make -C native/zero-c` after `cp -r` of the bind-mounted clone: FAILED. `cp: cannot create regular file ... Input/output error`, then `unexpected EOF`; the Docker daemon died (11m40s wasted). Cause: bind mount from the external SSD plus daemon crash. Environmental, not a Zerolang defect.
2. Restarted Docker Desktop (image was lost, re-pulled). Re-ran with `--tmpfs /work`: `error: expected declaration or statement at end of input` in `src/program_graph_check_gate.c`. The file was silently truncated by the bind-mount copy (container copy ended at line 49, host has 91 lines). Environmental.
3. Switched to a `tar` snapshot on the host internal disk. Failed once on macOS AppleDouble `._*` files (`skill-data/._agent.md: missing skill frontmatter`); fixed by `COPYFILE_DISABLE=1 tar --no-xattrs --no-mac-metadata`. Then the Docker daemon crashed again at ~3 min with the default `-Os` single `cc` invocation of all 130 sources (`error waiting for container: unexpected EOF`; VM has 4 GiB).
4. `make CFLAGS="-std=c11 -O0" -m 2g`: `cc: fatal error: Killed signal terminated program cc1` after 37 s (out of memory; tmpfs counted against the container memory limit).
5. SUCCESS: custom script, same sources, same embed scripts, but compiled per-file: `cc -std=c11 -O0 -DZERO_BUILD_HASH='"7e1a64d2"' -Iinclude -c <each src/*.c>` then one link. 85 s compile time, 39,790,072-byte binary. Generated: `embedded_runtime_sources.inc` 199,452 B, `embedded_skills.inc` 146,391 B, `embedded_stdlib_graph.inc` 2,116 B (via `node --experimental-strip-types scripts/embed-*.mts`).
   - Deviation from `make`: no `-Wall -Wextra -Wpedantic -Os` flags, per-file compile, build hash set manually. **This is an -O0 build. No performance numbers are reported or compared** (see Performance evidence).
   - First pass, not run: `pnpm run conformance`, `native:test`, `bench`, `evals`. Partly superseded by the second pass below.
6. **Second pass (2026-11-05, after Docker was moved to the SSD and restarted).** Host-side copy of the clone in `aqelith-lab/build/` (no in-container `cp` from the bind mount), image `node:22-bookworm` (gcc 12.2.0), `--memory=3g`, no tmpfs.
    - **Official build:** `make -C native/zero-c ZERO_BUILD_HASH=7e1a64d2` with the Makefile's own flags (`-std=c11 -Wall -Wextra -Wpedantic -Os`): exit 0, 103 s on linux/aarch64, with compiler warnings (e.g. a misleadingly indented `if` at `src/main.c:6372`). Same make on linux/x86_64 under emulation: exit 0, 219 s. `zero --version`: `zero 0.3.4 (build 7e1a64d2)`. This replaces the -O0 per-file build for functionality checks; timings still come from an emulated or VM-limited environment and are not comparable.
    - **Validation-gap finding re-checked on the -Os build** (fresh `zero init`, `addMain`, `addCheckWrite`, then `addLetLiteral fn="main" name="x" type="u32" value="notanumber"` and `appendStmt fn="main" stmt="check world.out.write(42)"`): both patches applied with exit 0 ("validated: check-equivalent"); `zero check` printed `ok`, exit 0; `zero check --json` reported `buildable:false` and `semanticDiagnosticsEnforced:false`. The finding is therefore not an artefact of the -O0 build. The op syntax (`name="x"`) was my reconstruction and was accepted.
    - **Native test script** (`ZERO_NATIVE_TEST_ALLOW_LOCAL=1 ZERO_NATIVE_TEST_SCOPE=fast bash scripts/test-native.sh`; the `fast` scope was my choice to bound time):
        - linux/aarch64: C smoke tests passed (`artifact finalization smoke ok`, `fs runtime smoke ok`) and the stdlib target matrix compiled `direct-checksum-helpers.graph` for `linux-musl-x64`, `darwin-arm64`, `win32-x64.exe` and `stdlib-target-neutral.graph` for `win32-x64.exe` and `linux-musl-x64`. The script then stopped at the first native case: `bin/zero run conformance/native/pass/stdlib-target-neutral.graph` failed with `BLD004: direct backend buildability does not support this MIR value` (expected "direct AArch64 ELF executable subset", actual `IR_VALUE_HTTP_REQUEST_BODY_WITHIN`). A conformance fixture in a `pass/` directory does not run on an AArch64 host.
        - linux/x86_64 (emulated): the same fixture runs, `zero run` exit 0. The script passed the preflight and 4 native cases (`examples/hello`, `add`, `generic-pair`, `fallibility`), then stopped with `cross target 'linux-musl-x64' requires a target-capable C compiler; pass --cc/ZERO_CC`.
        - linux/x86_64 (emulated) with `musl-tools` and `ZERO_CC=musl-gcc`: `musl-tools` was installed in a separate fetch-only image step, then the test script ran with `--network=none`. It exited 1 about 69 s in, right after the target matrix, with no error text in the log and no native case run. An earlier run of the same step with the network enabled gave the identical outcome and is not used as evidence, because it broke the isolation policy. The cause is not diagnosed, and I stopped after four runs.
    - **Result:** the test suite was NOT completed on any platform. Verified: build, smoke tests, target-matrix compilation, 4 native examples on x64, and the AArch64 backend gap. Not run: `pnpm run conformance`, `test:zero`, `bench`, `evals`, `extension:test`, the sanitizer script.

Experiments on the built binary (container, `--network=none`, `/tmp/t` scratch):

| Command | Result |
|---|---|
| `zero init` | ok; wrote `zero.toml`, `zero.graph` (610 B, binary, magic `ZRGBIN1`) |
| `zero patch --op addMain --op 'addCheckWrite fn="main" text="hello from zero\n"'` | ok; `graphHash: graph:5fce5563274fc6ab`, "12 nodes touched", "validated: check-equivalent" |
| `zero check` / `zero run` | `ok` / `hello from zero` (no `.0` file existed; compiled from `zero.graph` alone) |
| `zero query --fn main --handles` | lists nodes with ids like `#fn_main`, `#stmt_main_check_write_0`, resolved calls with `symbol:t@0.1.0/main::value.main/param.world` |
| `zero patch --expect-graph-hash graph:deadbeefdeadbeef ...` | exit 1, `graph hash precondition failed` with expected/actual; `zero.graph` sha256 unchanged before/after |
| `zero patch --op 'set node="#main_c" field="value" expect="WRONG" ...'` | exit 1, `patch field precondition failed` |
| `zero patch --op 'addLetLiteral ... type="u32" value="notanumber"'` | **exit 0**, `patch ok`, "validated: check-equivalent" (accepted a type-invalid literal) |
| `zero patch --op 'appendStmt fn="main" stmt="check world.out.write(42)"'` | **exit 0**, accepted |
| `zero check` after both | `ok` (exit 0). `zero check --json` shows `"ok":true` but `targetReadiness.buildable:false` with `BLD004 typed graph MIR integer literal is malformed`; and `"semanticDiagnosticsEnforced":false` |
| `zero fix --plan --json` | exit 1, reports BLD004, `fixSafety: requires-human-review` |
| `zero export`, text edit in `src/main.0`, `zero status` | `source-stale`, `projection validity: stale` |
| `zero run` on stale/edited projection | exit 1, `TYP002: let binding type does not match initializer` (the type error the graph path had accepted is caught only on the text path) |
| `zero verify-projection` | exit 1 `RGP006` with repair hints `zero import .` / `zero export .` |
| `ZERO_STALE=fail zero check` | exit 1 `RGP008` |
| `zero patch --expect-graph-hash graph:0000000000000000 ...` (second session) | exit 1; sha256 of `zero.graph` identical before/after (no write on rejection) |
| `zero patch --op 'addLetLiteral fn="nope" ...'`, `--op frobnicate` | exit 1, structured text error with expected/help; store unchanged |
| `zero patch --dry-run --json ...` | ok JSON with `originalGraphHash`, `patchedGraphHash`, per-op results |
| `zero skills`, `zero skills get graph` | 9 embedded topics served from the binary |
| Issue #435 repro: 60,000 `+` in `src/main.0`, `zero import .` | **exit 139 (SIGSEGV)**, reproduced on my -O0 build; issue reports it on the v0.3.4 release binary and on main |

## Feature audit

Classification vocabulary: IT = Implemented and tested; IE = Implemented but experimental; PI = Partially implemented; PL = Planned; MC = Marketing claim only; NV = Not verifiable. "Tested" means a test/conformance artifact exists in the repo; I did not run the suite (see Commands), so IT rests on artifact presence plus my own experiments where noted. The whole project self-labels experimental, so "IT" means "exists with tests", not "stable".

| Feature | Classification | Evidence |
|---|---|---|
| Graph is the compiler input (`zero.graph`, binary store, no `.0` needed to check/run) | IT | Experiment: check+run with no `.0` file. `skill-data/graph.md` "Source Boundary"; `conformance/program-graph/`; `native/zero-c/src/program_graph_store_binary.c` |
| `.0` text as projection; explicit `import`/`export`/`verify-projection`/`status` | IT | Experiment (RGP006, RGP008, status `source-stale`); `docs/articles/concepts/projections.md`; `program_graph_projection*.c` |
| Automatic refresh of stale store from edited text | IE | Documented in `skill-data/graph.md`; I saw refresh is refused when text introduces a diagnostic. Whole-file rewrites regenerate identities (documented) |
| Stable node identity across edits | PI | `native/zero-c/src/program_graph_node_id.c:134-165`: id = 32-bit truncated FNV-1a of (module identity, domain, kind, type, flags, owner id, edge kind, sibling order); collisions get 16-bit suffix (`:200-225`). Position-derived, not UUID/content-independent of position. Issue #348 (open) says ids survive content edits and, via collision handling, inserts/reorders; follow-up "diff/collision/multi-agent merge" PRs 2-8 still unbuilt |
| Stable symbol identity | PI | `program_graph_resolve.c:236-267`: `symbol:<module>::<kind>.<name>` derived from the name (non-alnum -> `_`); a rename changes the symbol id. Loop vars/match arms append node id. `type_id`/`effect_id` are FNV hashes (`type:b30331c589d91166`) |
| Semantic graph queries (`query --fn/--find/--refs/--calls/--node --depth`, `view`) | IT | Experiment output (resolved calls with `symbol:`, parents/children with edge orders); `program_graph_query*.c`; `conformance/program-graph` |
| Handle shorthand (`#55ae`, `#head..tail`, nearest-match on miss) | IT | `program_graph_handle.c:20-60`; `skill-data/graph.md` |
| Structured graph patches (typed ops, node-level `set/insert/replace/delete/rename`, `replaceFunctionBody`, `upsertFunction`) | IT | `program_graph_patch*.c` (~4.3k lines); experiments; `zero patch --op help` lists ~30 ops |
| Patch dry-run / check-only / JSON result | IT | Experiment (`--dry-run --json`); `program_graph_patch.c` |
| Structural rewrite-by-example (`--rewrite ... --to ... --apply`) | IE | `skill-data/graph.md`, `program_graph_rewrite.c`; added in 0.3.4; not exercised by me |
| Stale patch detection: graph-hash precondition (`--expect-graph-hash`, `expect graphHash`) | IT | Experiment (GPH002, store sha256 unchanged); `program_graph_patch.c:974-977`. Opt-in: a patch without the precondition is not rejected as stale |
| Stale detection: node-hash and field-value preconditions (`expect=`, `nodehash:`) | IT | Experiment (GPH005 field precondition); `program_graph_patch_ops.c:987,1208,1317` |
| Stale detection: projection vs store (content-classified, not mtime) | IT | Experiment; CHANGELOG 0.3.3 "classifies source/store sync by content instead of mtimes" |
| "Compiler validation before mutation" (patch fails before store write if invalid) | PI | Shape/structure validation works (`z_program_graph_validate` called at `program_graph_patch.c:986`; failed patches left store byte-identical). BUT my experiments: type-invalid `let x: u32 = "notanumber"` and `write(42)` were accepted by `patch` and `check` returned `ok`; real errors surfaced only at BLD004 in readiness or TYP002 via the text path. `check --json` states `semanticDiagnosticsEnforced:false`, `semanticDiagnosticsAuthority:"stored-typed-graph-facts"`. Docs claim "type errors fail before the store is written" (`docs/articles/concepts/graph-architecture.md`). Contradicted on this build for these inputs (caveat: -O0 build, op-level literal misuse) |
| Atomic store write | IT | `fs.c:230-276` temp file `O_EXCL` + `rename` |
| Concurrent-writer protection (lock) | NV / not found | No `flock`/lock around patch in `program_graph_store.c`/`fs.c` (grep). Hash precondition is the only guard. Multi-agent merge is PL (#348) |
| Agent CLI (`init/query/view/patch/check/test/run/diff/explain/fix --plan/skills/...`) | IT | `docs/articles/cli-reference.md`; experiments |
| Version-matched agent skills embedded in binary (`zero skills get <topic> [--topic]`) | IT | Experiment (9 topics); `scripts/embed-skill-data.mts`, `native/zero-c/src/embedded_skills.inc` (146 KB, generated at build); `skills/zero/SKILL.md` is a thin stub (`npx skills add vercel-labs/zerolang`) |
| Structured diagnostics (stable codes, `--json`, `expected/actual/help/fixSafety/repair/related`) | IT | Experiment (BLD004 JSON); `skill-data/diagnostics.md`; `zero explain <code>` works. Gaps: #111 `zero explain PAR100` unknown (open), #173 stabilizing contracts (open), `ok:true` alongside `buildable:false` |
| Typed repair plans / fix safety levels | IE | `zero fix --plan --json` returned a plan with `fixSafety`; levels documented; most results `requires-human-review` in my run |
| Compile-time meta / reflection sandbox | IE | `check --json` `compileTime` block: fs/net/env/process denied, limits `maxDepth 64`, `maxSteps 1024`, `stringBytes 127` |
| Explicit capabilities / `World` effects | IE | `docs/articles/target-capabilities.md`; `check --json` `uncheckedSurfaces`: "host filesystem/process/network effects: capability-gated-not-sandboxed" |
| Memory safety model (bounds checked, borrow/move checks, 128 KiB frame limit MEM003) | IE | `safetyFacts` JSON; `skill-data/diagnostics.md`; #318 (18 confirmed bugs incl. borrow soundness, wrong-code), #316, #317 |
| Native codegen without LLVM/libc (direct emitters: ELF/Mach-O/COFF x64+aarch64) | IE | `native/zero-c/src/emit_*.c`, `aarch64_direct.c`; open backend gaps #96, #437, #438, #230 |
| LLVM IR backend | IE | CHANGELOG 0.3.0 "explicit experimental LLVM backend" |
| Targets | PI | `targets.manifest`: darwin-arm64/x64, linux(-musl)-x64/arm64, win32-x64/arm64 (+ wasm mentioned in issues #95). Windows/Darwin direct backend gaps (#437, #440) |
| Package manager (local path deps, lockfile `zero-lock-v1`, resolver `exact-version-or-path`) | PI | `docs/articles/package-manifest.md`: "registry references recorded as metadata without remote fetches"; `check --json` `resolver.remoteFetch:false` |
| Stdlib (34 graph-backed modules: json, http, regex, time, crypto, fs, ...) | IE | `std/*.graph` (34), ~18.7k lines of `.0` projections; #326 crypto roadmap; #428 test+std bug |
| Token-efficiency / fewer tool calls than text editing | MC | No benchmark in repo comparing graph vs text edit cost. `evals/` harness exists (needs Vercel Sandbox + AI Gateway); no published results found. Site gives no numbers |
| Low memory / fast startup / fast builds | NV | Goals only (README "Runtime Goals"); `scripts/bench.mts` and `graph-build-perf.mts` exist with budgets, but I did not run them; no published numbers |
| "Sub-10 KiB native binaries" (zerolang.app) | NV | Not tested; unofficial-looking site |
| Frontier-model first-try success | MC / negative data | Issue #104: 0/60 attempts by gpt-5/4o/4o-mini without examples (`use std`, `fn` vs `fun`, PAR100/IMP001); that was early syntax (May 2026) and syntax churn is noted in #290 |
| Multi-agent concurrent editing / merge | PL | Issue #348 (RFC, open) |
| Security hardening | PI | CHANGELOG 0.3.1 (argv-based exec, atomic outputs, checked reads); fuzzing report #435 (SIGSEGV, reproduced) |

## Architecture notes

- Compiler: one C11 program, ~128k lines in `native/zero-c` (`main.c` 17k, `checker.c` 13k, `program_graph_mir.c` 7k, `ir.c` 6.8k). No dependency on LLVM/libc for default direct backends; Node/pnpm only for dev tooling (docs site, evals, embed scripts). Skills, stdlib graphs and runtime sources are embedded in the binary via generated `.inc` files (`Makefile:26-37`).
- Program graph (`program_graph.h`): nodes (module, import, decl kinds, params, fields, blocks, statements, expressions, TypeRef, EffectRef, ErrorVariant, ~46 kinds), typed edges (`kind`, `target` node|symbol|type|effect, `order`). Node has `symbol_id`, `type_id`, `effect_id`, `node_hash`, line/column (source map into the projection). Validation ladder: decoded -> shape-valid -> resolved -> typed -> effect-checked -> buildable.
- Hashes: graph hash, node hash, ids and cache keys use FNV-1a 64 (`program_graph_identity.c:84-100`; "fnv1a64" in JSON). Fine for staleness detection, not tamper evidence.
- Storage: `zero.graph` binary (`ZRGBIN1` magic) or text; atomic write via temp+rename. Repository metadata includes compiler-oriented tables (symbol, type, effect, capability, ownership, resource, node, edge, sourceMap). Binary store is compiler-build-specific (RGP009: "written by a different zero build").
- Patch pipeline (`program_graph_patch.c`): parse ops (CLI `--op` or `zero-program-graph-patch v1` text, with `end`-terminated bodies) -> check optional `expect graphHash` -> apply each op, re-finalizing identities after each (`z_program_graph_finalize_identities`) -> compact ordered edges -> structural validation -> save. Body ops accept canonical projection syntax rows, so agents write code-like text inside a structured envelope (important for the AQELITH question below).
- Projection round trip: `export` writes `.0`; text edits are detected by content comparison; compiler commands auto-reconcile by re-parsing text and matching old nodes to preserve handles ("structurally closest match", ties fail with RGP007).
- Package/module architecture: `zero.toml`/`zero.json` manifest + graph + `src/*.0` projections; stdlib shipped as graph modules and merged into the program graph at build (`program_graph_std_merge.c`) with pruning; local path dependencies only; lockfile generated under `.zero/package-locks/`. Build caches keyed on graph hash, stdlib graph hash, module/node hashes, type/symbol facts, target, profile, compiler version.
- Test/CI: `conformance/` (agent-surface, program-graph, diagnostics, run, ...), `native/zero-c/tests/*.c`, `tests/zero-cli.test.ts`, `evals/`; `.github/workflows/ci.yml`. I ran only part of the native test script (see Commands executed, item 6); the rest was not run.
- Bus factor: 1187 of 1188 commits by one author; AGENTS.md says "Do not preserve legacy behavior by default", i.e. breaking changes are policy.

## Security considerations

- The project says: "Expect breaking changes, rough edges, and security issues" (README); AGENTS.md: "Security vulnerabilities should be expected."
- Memory-safety bug class in the compiler itself (C, no sanitizers in default build): #435 unbounded recursion SIGSEGV on 60 KB of `+`, reproduced by me (exit 139) through `zero import`. A hostile `.0` projection (e.g. from a PR) can crash the compiler/CI. Found by AFL++ with ASan/UBSan; #318 lists 18 more reproduced bugs (wrong-code, borrow soundness). `scripts/sanitizer-smoke.sh` exists.
- Installer is `curl | bash` from zerolang.ai (`docs/public/install.sh`); it verifies SHA-256 against `CHECKSUMS.txt` downloaded from the same GitHub release URL, so it protects against corruption, not against a compromised release. No signature/cosign/attestation in the install script (grep). I did not audit `release.yml` further.
- Capability model: `check --json` itself states host fs/process/network effects are "capability-gated-not-sandboxed"; C imports "externally-trusted". Graph hashes (FNV-1a 64) are not an integrity mechanism; any process with write access to `zero.graph` can forge a matching hash.
- `zero.graph` is a binary artifact in review: reviewers must trust `export`/`diff`; a malicious graph can differ from its checked-in projection unless `verify-projection` is enforced in CI (README suggests it, nothing forces it).
- Compile-time evaluator is bounded and denies fs/net/env/process (JSON `compileTime.sandbox`).
- Dev-environment guidance: run in isolated workspaces (README warning). My builds used `--network=none` containers.

## Performance evidence

- No performance numbers were measured by me that are fit for comparison: my binary is an `-O0` per-file build, not the project's release `-Os` build, so any timing would be invalid to compare with their budgets (the instructions forbid mixing). I record only: my compile of the compiler took 85 s at -O0 inside a throttled Docker VM, and each CLI call in my experiments returned effectively instantly (the experiment container script ran about 3 s wall time in total, including ~40 commands).
- Project-reported (not verified): `docs/articles/benchmarks.md` says benchmarks "are regression signals ... not broad marketing claims"; metrics are `buildMs`, `runMs`, `artifactBytes`, `peakRssBytes`. `scripts/graph-build-perf.mts` defines budgets: load 50 ms, validate 50 ms, resolve 50 ms, check 50 ms, lower 100 ms, cache 25 ms (also echoed in `check --json` `performance.budgets`). CHANGELOG claims "roughly 12x faster" `zero import` on large programs (0.3.2) and patch loop "0.2s validation" (`skill-data/graph.md`); these are unverified.
- zerolang.ai gives no numbers; zerolang.app claims "sub-10 KiB native binaries" (unverified, unofficial-looking).
- No benchmark found comparing token cost or tool-call count of graph patching vs text editing. That is the central claim of the project and it is unsubstantiated in public artifacts I could find (`evals/` can measure it but results are not committed).

## Known limitations

- Validation gap: `patch`/`check` accepted type-invalid values on this build (see Feature audit); `check` returns `ok` while the same JSON says `buildable:false`. Semantic checker authority is "stored-typed-graph facts", not enforced.
- Experimental across the board; syntax and CLI break between versions (#290, #104: `fn` vs `fun`). v0.3.4 skill/data are matched to the binary, but stores are not portable across builds (RGP009).
- Graph ids: 32-bit position-derived hashes with collision suffixes; handle stability on sibling insert/reorder relies on collision logic (#348 open). Symbol ids are name-derived and not rename-stable.
- Open correctness issues: #435 (crash), #436 (fallible Void returns spurious error), #439/#437/#438 (Windows/Mach-O backend gaps), #318 (18 audited bugs), #428 (tests cannot reach `std.*` outside `std.testing`), #425 (module hash comparison), #442/#441 (patch/`--replace-in-fn` tips).
- No concurrent/multi-agent edit story, no registry, local path dependencies only, no lock around the store.
- Bus factor 1; ~7 weeks since last commit, ~5 months since last release at inspection. Repo honesty is good (README/AGENTS warnings), but a community-run site (`zerolang.app`) repeats claims with numbers not in the repo.
- Human trust: issue #181 ("Low code quality. Don't run this on your machine"): no C contract docs, string comparisons in IR; maintainer replied by making the safety warning prominent. `native/zero-c/CONTRACTS.md` was later added for pointer access effects.
- Gaps in my audit: the full test suite was not completed on any platform and benchmarks were not run (see Commands executed, item 6); the published release binary (`zero-linux-x64`) was not downloaded or compared, but the official `make` build (-Os, same version) was built and reproduced the validation-gap finding; Windows/macOS paths not exercised; platforms: linux/aarch64 native and linux/x86_64 under emulation.
- Observed here: a conformance `pass/` fixture (`stdlib-target-neutral.graph`) does not run on an AArch64 host (BLD004) but runs on x86_64.

## Ideas worth adopting for AQELITH

Mandatory caveat first: **the graph-vs-human-readable-source question.** Zerolang's evidence is mixed. It chooses the graph as truth and keeps text only as a projection; in practice it had to build an entire reconciliation layer (stale/diverged/ambiguous states, RGP006/007/008, handle-preserving import, whole-file-rewrite acceptance, content-based sync) because humans and agents both want text. Even its patch language embeds canonical source rows (`replaceFunctionBody`, `upsertFunction`, `--replace-in-fn` with Edit semantics), and its own skill says to prefer these over node-level ops: agents gravitate to text-shaped edits inside a checked envelope. The claimed benefit (fewer tool calls/tokens, fewer text-edit failures) is unmeasured publicly. Costs observed: binary store tied to compiler build, review requires a second artifact and a drift gate, a hash that is not integrity, and validation seeming weaker on the graph path than on the text path (type errors caught only when the text projection was re-parsed). AQELITH decision criteria: (a) text-canonical + typed index/patch layer, vs (b) graph-canonical + generated text. Zerolang is evidence that (b) pushes complexity into sync tooling and makes "validated before mutation" harder to guarantee; it is not evidence that (b) improves agent success. Treat the question as open and run a controlled eval before committing.

Adopt (adapting to either representation):

1. Optimistic concurrency preconditions on every mutation: `expect graphHash` plus per-node `nodehash:` and per-field `expect=` values, each failing with expected/actual and leaving the store byte-identical (verified). Make them mandatory-by-default in AQELITH rather than opt-in.
2. Dry-run/check-only patches that return JSON with before/after hashes and per-op results.
3. Handle ergonomics: short unique-prefix handles and "nearest existing handle" error on a miss; handle-annotated views (`view --handles`) that show the text and its ids together.
4. Version-matched skills served by the binary (`skills get <topic> --topic <section>`), thin external stub. Excellent for agent drift avoidance; cheap to copy.
5. Diagnostics as repair contracts: stable codes, expected/actual, `fixSafety` levels (format-only, behavior-preserving, api-changing, target-changing, requires-human-review), `explain <code>`.
6. Validation-ladder vocabulary (decoded, shape-valid, resolved, typed, effect-checked, buildable) reported in `status`; AQELITH should *gate mutations on the strongest level* (Zerolang's gate stops at shape-valid, which is the observed hole).
7. Explicit sync commands plus a no-write drift check (`verify-projection`) if AQELITH ever keeps two representations.
8. Atomic write via temp+rename; consider adding a file lock (Zerolang lacks one).
9. Structural rewrite-by-example (`--rewrite '$A == 0' --to ...`) with dry-run listing every site: strong for cross-cutting agent edits.

Avoid:

- Position-derived 32-bit ids and name-derived symbol ids as the "stable identity": use persistent opaque ids (UUID/ULID-like) assigned at creation and carried through moves/renames; keep a content hash separate for staleness.
- Non-cryptographic hashes where integrity matters; use a collision-resistant hash for graph/artifact identity if signatures or multi-agent merge are planned.
- Declaring "ok" when the build-readiness report says not buildable; one verdict should be authoritative.
- Shipping a C compiler without recursion/size limits on parsers (#435); enforce depth/size limits and fuzz from day one.
- Claiming token/memory/speed wins without committed benchmarks.

## Confidence

| Section | Confidence | Why |
|---|---|---|
| Sources / Inspected version | High | Commit, tag, dates verified from git and `gh`; sites fetched (zerolang.app operator unknown) |
| Commands executed | High | Verbatim outcomes recorded, incl. five failed builds; environment-induced failures were separated from project defects |
| Feature audit | Medium | Source and experiments support most rows, but the test suite was not completed (partial native script only), and the published release binary was not compared; the validation-gap finding is reproduced on the official -Os build of the same version, though op usage was reconstructed from the report's abbreviated syntax |
| Architecture notes | Medium-High | Core graph/ID/patch/storage code read directly; large parts (MIR, backends, reconcile internals) skimmed via docs and file sizes only |
| Security considerations | Medium | #435 independently reproduced; installer read; release workflow, supply chain, and fuzzing coverage not audited |
| Performance evidence | Low | No valid measurements (debug build); project numbers unverified; no public graph-vs-text benchmark found |
| Known limitations | Medium-High | Combines my experiments with issue-tracker evidence; issues are single-reporter claims unless I reproduced them (#435 reproduced) |
| Ideas worth adopting | Medium | Sound where grounded in observed behavior; the graph-vs-text conclusion is deliberately left open pending an eval |
