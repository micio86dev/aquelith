# Competitor audit: Edict (Sowiedu/Edict)

Audit date: 2026-11-05. Auditor: research agent for AQELITH. Method: identity search, full clone, source/docs/tests/CI/issue/release reading. The first pass could not build (see Commands executed); a second pass ran `npm ci`, `npm run build` and `npm test` successfully (item 4 under Docker). "Implemented and tested" labels rest on reading the tests plus the passing local suite; features needing external services or a live account were not exercised.

## Sources

- Repo (authoritative): https://github.com/Sowiedu/Edict (MIT, created 2026-02-24)
- npm package: `edict-lang` (package.json name; `mcpName` io.github.Sowiedu/edict)
- Releases: https://github.com/Sowiedu/Edict/releases (30 releases; latest v1.23.2, 2026-04-15)
- Third-party directory listings (secondary only, not relied on for facts): https://mcp.so/ja/servers/edict, https://mcpmarket.com/server/edict, https://agentseal.org/mcp/edict-lang (scanner score 40/100, not reproduced), https://glama.ai/mcp/servers/Sowiedu/Edict
- Local clone: `<local-disk>/aqelith-lab/edict@2dbed48`
- Key files read: `README.md`, `ROADMAP.md`, `FEATURE_SPEC.md`, `AGENTS.md`, `SECURITY.md`, `UASF.md`, `CHANGELOG.md`, `docs/contract-coverage.md`, `docs/self-hosting-status.md`, `src/checker/check.ts`, `src/effects/effect-check.ts`, `src/contracts/{verify,translate}.ts`, `src/contracts/solver/index.ts`, `src/patch/apply.ts`, `src/codegen/{runner,node-host-adapter}.ts`, `src/builtins/domains/http.ts`, `benchmarks/error-recovery/*`, `.github/workflows/ci.yml`.

## Identity verification

- Searches (2026-11-05, WebSearch): (1) "Edict programming language for AI agents structured AST type effect checking Z3 SMT WASM MCP github"; (2) '"Edict" language AI agents JSON AST WebAssembly "MCP" formal verification'. Both converge on a single project, GitHub `Sowiedu/Edict`. No competing candidate matching the feature set appeared; other hits were MCP-security papers (unrelated) and directory mirrors of the same repo.
- Verified from the repo itself: `package.json` name `edict-lang`, description "A programming language for AI agents. JSON AST in, WASM out. MCP server included. Statically typed, effect-tracked, contract-verified"; keywords include z3, smt, wasm, mcp, json-ast; `src/` contains the pipeline directories (validator, resolver, checker, effects, contracts, codegen, mcp). GitHub API: owner Sowiedu, MIT, created 2026-02-24, 13 stars, 2 forks.
- Name collisions NOT matched: the many unrelated "Edict" projects (e.g. the EDICT image-diffusion inversion paper, Edict game/legal libraries) do not claim a JSON-AST/WASM/MCP language. Rejected on feature mismatch; not individually enumerated beyond this.
- Caveat: sole maintainer ("Patrick Probst"/"Patrick": 292 of 346 commits; remainder bots). No institutional backing found.

## Inspected version

- Commit `2dbed48beda84d43e746ebec8bc3d98bcdcc63b0` (2026-10-25, "Merge pull request #251 from Sowiedu/fix/node-brotli-cve-2026-111497"), default branch HEAD at clone time. Short form `2dbed48`.
- package.json version `1.23.2`; latest git tag/release v1.23.2 (2026-04-15). HEAD is ~6 months of commits past the last release (dependency/CVE maintenance visible; GitHub pushed_at 2026-10-31).
- 346 commits, first commit 2026-02-24. Inspection date 2026-11-05.

## Commands executed

Host (read-only analysis of untrusted code; no project code executed on host):
- `git clone https://github.com/Sowiedu/Edict.git` then `git rev-parse --short HEAD` -> `2dbed48`; directory renamed to `edict@2dbed48`. OK (475 files).
- `gh api repos/Sowiedu/Edict` -> stars 13, forks 2, open_issues 6, MIT. `gh api .../releases` -> 30 releases. `gh issue list --state all` -> 167 CLOSED returned; the 6 "open_issues" API count includes PRs (the open-issue listing printed nothing, i.e. 0 open issues visible).
- Static greps/line counts: ~30.7 kLOC TypeScript in `src/`; 136 test files; 2346 literal `it(`/`test(` call sites (README claims 2675 tests; the second-pass run reported 2675 tests: 2674 passed, 1 skipped, so the README figure is accurate).

Docker (all with `--network=none` NOT applicable to the fetch step; fetch step used network, as permitted):
1. `docker run --rm -v "$PWD":/w -w /w node:22-slim sh -c 'node -v; npm -v; npm ci --ignore-scripts'` (network, fetch step). Ran >10 minutes with no output, left node_modules incomplete (212 dirs, no `.bin`). Root cause identified afterwards: the host system volume was at 100% (140 MiB free), which produced "No space left on device" on every scratch write, and Docker Desktop then reported "Docker Desktop is unable to start". RESULT: FAILED (environment).
2. Retry after cleaning my partial `node_modules` (npm cache redirected to the SSD, `--tmpfs /tmp`): first attempt "Error response from daemon: Docker Desktop is unable to start"; second attempt hung ~6 min with zero output; afterwards `docker ps` / `docker version` print only `EOF` (daemon unresponsive). Disk had recovered to ~3 GiB free but the daemon did not. Other concurrent audits share the same Docker VM and host disk, which likely contributed.
3. First pass: planned and NOT run (no working Docker): `npm run typecheck`, `npx vitest run --exclude tests/mcp/...`, `npm run build`, `npm run validate-examples`. Superseded for build and tests by item 4.
4. **Second pass (2026-11-05, after Docker was moved to the SSD and restarted).** On a copy of the clone in `aqelith-lab/build/`, image `node:22-bookworm` (Node 22.23.4, npm 10.9.9), `--memory=3g`:
    - Isolation: the first run of this pass used `npm ci` with install scripts and network on inside the container, which deviated from the policy (fetch only while online, run code with `--network=none`). It was redone: `npm ci --ignore-scripts` (network on, fetch only, 95 s, exit 0), then `npm rebuild` (`--network=none`, exit 0), then build and test with `--network=none`. The results below are from the compliant rerun and were identical to the first run. npm printed an `npm audit` warning during the first run; the count was not captured, so no figure is reported.
    - `npm run build` (`--network=none`; runs schema generation, `tsc`, browser and QuickJS bundles): exit 0, 16 s in the compliant rerun (13 s in the first run). One esbuild warning: `import.meta` is empty in the IIFE QuickJS full bundle (`dist/contracts/verify.js:176`).
    - `npm test` (`--network=none`, vitest, whole suite, including the MCP and benchmark tests): exit 0, 136 files passed, 2674 tests passed, 1 skipped, 115 s in the compliant rerun (136 s in the first run).
    - Measured artifact sizes: `dist/browser.bundle.js` 355,511 B; `dist/browser-full.bundle.js` 899,520 B; `dist/z3-built.wasm` 33,704,614 B; `dist/edict-quickjs-check.js` 381,781 B; `dist/edict-quickjs-full.js` 954,666 B. These were not reconciled with the README browser table (318 KB check-only, about 14 MB full with Z3), because it is unclear which files and which compression that table refers to.
    - Not run: `npm run validate-examples`, any Edict program through the MCP server, the Docker image, the Cloudflare deploy path, and the benchmarks as performance measurements (the error-recovery benchmark ran only as a test, and it uses the mock repairer described above).

Outcome: first pass produced no build or test run (superseded by item 4: build and the full test suite pass locally); no Edict program was executed through the CLI or MCP server and no performance was measured. Findings come from reading code, docs, tests, CI config and GitHub metadata, plus the second-pass results. CI (`.github/workflows/ci.yml`) runs `npm ci`, `typecheck` and tests on Node 20 and 22; I did not inspect CI run results, so "CI is green" is Not verified.

## Feature audit

Classification key: IT = Implemented and tested; IE = Implemented but experimental; PI = Partially implemented; PL = Planned; MC = Marketing claim only; NV = Not verifiable. "Tested" means test files exist for it; the whole suite also passed in the second pass (Commands executed, item 4), but I did not map each row to specific tests. ABS = Absent: verified not present and not claimed by the project; used only for comparison rows and outside the six mandated classes.

| Feature | Classification | Evidence |
|---|---|---|
| JSON AST as canonical program format (no parser/text syntax) | IT | `README.md:12`, `src/ast/nodes.ts`, `schema/edict.schema.json`, `src/validator/`, `tests/validator/` |
| Stable per-node `id`, duplicate-id rejection | IT | `ROADMAP.md` Phase 1 edge cases; `tests/validator/invalid-programs.test.ts` |
| Structured, typed errors with repair context (nodeId, expected type, candidates, counterexample) | IT | `src/errors/structured-errors.ts`, `src/errors/error-catalog.ts`, `tests/errors/`, MCP `edict_errors` |
| Name resolution with Levenshtein suggestions | IT | `src/resolver/`, `tests/resolver/` |
| Static type checker (bidirectional, Int/Int64/Float/String/Bool/Array/Option/Result/records/enums/fn types) | IT | `src/checker/check.ts` (1364 lines), `tests/checker/check.test.ts` |
| Polymorphic containers / type-var unification | IT | `CHANGELOG.md` v1.23.0; `src/checker/unify.ts`; `tests/checker/` |
| Unit types (`currency<usd>`) mismatch detection | IT | `src/checker/check.ts:424-432`, `tests/checker/unit-types.test.ts` |
| Unit arithmetic composition (`distance/time = speed<m/s>`) | MC | Spec claims it (`FEATURE_SPEC.md:91`) but `*`,`/`,`%` require `typesEqual` operands (`src/checker/check.ts:~395-404`); no composition rule found |
| Refinement types `{i: Int \| i > 0}` verified by Z3 | PI | Type form exists (`README.md` Type System); `docs/contract-coverage.md` shows String/Array params and call expressions are `undecidable` |
| Effect system `pure/reads/writes/io/fails` with call-graph propagation | IT | `src/effects/effect-check.ts:47`, `src/effects/call-graph.ts`, `tests/effects/` |
| Effect checks across imports | PI | Imported functions are "effect-opaque"; only an INFO diagnostic is emitted (`src/effects/effect-check.ts:~55-62`) |
| Effect polymorphism (effect variables) | IT | `ROADMAP.md` (#94 done); `src/checker/`, effect tests |
| Approval gates propagating through call chains | IT | `src/ast/nodes.ts:90-102,191`; `approvalPropagationMissing` in `effect-check.ts`; `examples/approval-gates.edict.json` |
| Contract (pre/post) verification via Z3 with counterexamples | IT | `src/contracts/verify.ts:1-12,532-575`, `tests/contracts/` |
| Z3 timeout handling (5 s per query, 30 s worker) and verification cache | IT | `src/contracts/verify.ts:38-39,306-315`, `hash.ts` |
| Callsite precondition checking | IT | `src/contracts/verify.ts:~780-840`; `docs/contract-coverage.md` tier T6 |
| Quantifiers `forall/exists` in contracts | PI | `translate.ts:113-114`; tier T7 4/6 proven; QF-LIA fallback solver does NOT support them (`src/contracts/solver/index.ts:21-23`) |
| Contract coverage on corpus | IE | Self-reported baseline: 38/53 proven (71.7%), 13 counterexample, 2 undecidable, 2 skipped, on 55 contracts as of v1.22.1+ (`docs/contract-coverage.md:5-13`); not re-run |
| Built-in pure-JS QF-LIA solver (Z3-free) | IE | `src/contracts/solver/solver.ts` (401 lines); "evaluation-based" per `solver/index.ts:3-4`; soundness vs Z3 not verified by me |
| Test generation from verified contracts | IT | `src/contracts/generate-tests.ts`, `tests/contracts/` |
| WASM codegen (own pure-JS encoder, via mid-level IR) | IT | `src/codegen/wasm-encoder.ts` (1221 lines), `src/ir/`, `tests/codegen/` |
| WASM execution in Node with limits (timeout, memory ceiling) | IT | `src/codegen/runner.ts:67-84`, worker-thread run at `runner.ts:145` |
| Pure-JS WASM interpreter (no WebAssembly API) | IE | `src/codegen/wasm-interpreter.ts`; "stack-based MVP interpreter" per `docs/self-hosting-status.md` |
| Deterministic record/replay of host calls | IT | `src/codegen/recording-adapter.ts`, `replay-adapter.ts`, MCP `edict_replay`, tests |
| Host adapters (Node, Browser, Deno, Cloudflare) | IE | `src/codegen/*-host-adapter.ts`; only Node and browser clearly tested (`tests/browser`, `tests/deploy`) |
| MCP server, 22 tools, resources, prompts | IT | 22 tool files in `src/mcp/tools/` (count verified); `tests/mcp/integration.test.ts` (in-process MCP SDK client over HTTP) |
| MCP stdio and HTTP transports, Docker image (amd64/arm64) | IT | `README.md` Docker section, `.github/workflows/docker.yml`; image not pulled |
| Agent-oriented editing: `edict_patch` by nodeId (replace/delete/insert), then re-check | IT | `src/patch/apply.ts:19-24,54-80`, `src/mcp/handlers/patch.ts`, `tests/` patch suite |
| Incremental checking (dependency graph + diff) | IT | `src/incremental/`, `tests/incremental/` |
| Compact (token-efficient) AST format | IT | `src/compact/expand.ts`, `tests/compact/expand.test.ts` |
| Schema versioning and auto-migration | IT | `src/migration/`, `tests/migration/` |
| Fragment composition | IT | `src/compose/compose.ts`, `edict_compose` |
| Multi-module imports | PI | `src/multi-module.ts`; imports are effect-opaque; WASM external modules via `externalModules` (base64) |
| Lint (non-blocking warnings) | IT | `src/lint/`, `edict_lint` |
| Browser build (318 KB check-only; ~14 MB full w/ Z3) | IT | `README.md` Browser table; `tests/browser`; sizes self-reported |
| Self-hosting inside QuickJS-WASM | IE | `docs/self-hosting-status.md` (dated 2026-03-19); contract phase "Partial"; 3.7x slowdown self-reported; `quickjs-emscripten` is an optional peer dep |
| UASF portable skill packaging and import | IE | `UASF.md`, `src/skills/`, `edict_export`/`edict_import_skill`; no evidence of an external ecosystem |
| Edge deploy to Cloudflare Workers | IE | `src/deploy/`, `edict_deploy`, `edict_invoke`; needs live account; not verifiable offline |
| Human-readable syntax / pretty printer | ABS | README: "no human-readable syntax"; no decompile/pretty-print module found (grep of `src/`) |
| Agent error-recovery benchmark | IE | `benchmarks/error-recovery/results.json`: 30-entry corpus, 76.7% recovery, but strategy is `mock-deterministic` (a hand-written repair function), NOT an LLM (`runner.ts:72-74`) |
| LLM-authoring success rate / token cost / latency claims | NV | No LLM-in-the-loop benchmark in repo |
| "Zero ambiguity", "defense-in-depth" sandbox security | MC (partly) | WASM sandbox is real; see Security considerations for gaps |
| Provenance/confidence/freshness types | IE | `tests/checker/provenance-types.test.ts`, `confidence-types.test.ts`, `freshness-types.test.ts`; not described in README, design intent unclear |

## Architecture notes

- Pipeline (README/ROADMAP, matches directory layout): schema validator -> name resolver -> bidirectional type checker -> effect checker -> contract verifier (Z3 or built-in QF-LIA) -> IR lowering/optimization -> own WASM encoder -> runner (Node `WebAssembly`, or pure-JS interpreter). Every phase emits `StructuredError` JSON; checking short-circuits on earlier-phase failure.
- Edict ships its own WASM encoder (replaced binaryen, issue #198) and its own QF-LIA solver (issues #205-#208) to enable QuickJS/browser self-hosting. This trades Z3 completeness for portability: quantified/array/string contracts degrade to `undecidable_predicate`.
- Z3 runs in a worker thread so the MCP server stays responsive; results are hash-cached by contract content (`verify.ts:306-315`, `hash.ts`).
- Agent loop is the product: `edict_schema` -> `edict_check` -> fix via `edict_patch` (node-id addressed ops, deep-cloned AST, `apply.ts:50-53`) -> `edict_compile` -> `edict_run`. A "minimal schema" resource (`edict://schema/minimal`) and compact format target token cost.
- Effects are a flat 5-label set, not row-polymorphic capabilities; the host can inspect them before running. Approval gates add a human-in-the-loop marker that propagates through callers.
- Persistence/identity: programs are `.edict.json` files; node ids are author-assigned strings (validated unique), not content hashes.
- Code size: ~30.7 kLOC TypeScript, 136 test files. Single-language (TypeScript/Node >= 20) dependency base: `@modelcontextprotocol/sdk`, `z3-solver`, `zod`.

## Security considerations

All from reading code; nothing exploited or executed.
- Sandbox model: compiled WASM has no ambient authority; I/O only via `EdictHostAdapter` (`runner.ts`, `host-helpers.ts`). `RunLimits` provides timeout (default 15 s), memory ceiling (default 1 MB), sandbox dir.
- File sandbox check is lexical: `validateSandboxPath` uses `path.resolve` + `relative` (`src/codegen/node-host-adapter.ts:26-33`). No `realpath` call anywhere in `src/codegen` (grep). A symlink inside the sandbox dir pointing outside would pass the check. Hypothesis, untested.
- HTTP allow-list is opt-in: with `allowedHosts` unset, all hosts are allowed (`src/builtins/domains/http.ts:10-15`, `README` RunLimits comment). Only hostname is compared. SSRF to internal addresses is possible by default for any program with `io` effect.
- HTTP is performed by spawning a child Node process with the FULL parent environment (`...process.env`, `node-host-adapter.ts:65-69`) plus the request in env vars; the child script is static, but the child inherits all secrets in the server's environment. Low direct risk (script is fixed) but poor hygiene.
- Env vars are exposed to programs as a host capability (README "environment variables"); no allow-list seen.
- Effect checking is advisory to the host: effects are declared by the author and enforced by the checker, but at runtime the host adapter is what limits capability. Imported modules are effect-opaque, so cross-module effect claims are unchecked.
- Z3 DoS surface is acknowledged in `SECURITY.md` (unbounded Z3 computation); mitigated by 5 s solver timeout and 30 s worker timeout (`verify.ts:38-39`).
- MCP server accepts arbitrary ASTs and `edict_deploy` can push to Cloudflare with credentials; `edict_import_skill` executes third-party packaged WASM. Trust model for imported skills (`UASF.md`) not audited in depth; there is no signature verification evidence found (not searched exhaustively).
- `SECURITY.md` lists "1.8.x" as the only supported version while the package is 1.23.2: stale policy. Fuzz tests exist (`tests/fuzz/`: validate, check, compile, stress).
- Third-party scanner score (agentseal 40/100) is unverified and unexplained; not relied on.
- Supply chain: recent merge is a Node/brotli CVE fix (#251), `dependabot` is active (22 bot commits).

## Performance evidence

- No performance numbers measured by me. Build took 13 s after `npm ci` (99 s) in a 4 GiB Docker VM; this is not a compiler speed claim. Artifact sizes are listed in Commands executed, item 4.
- Self-reported only: QuickJS bundles 373 KB (check) / 932 KB (full), ~3.7x slower than native Node (`README.md`; `docs/quickjs-feasibility-report.md`); browser bundles 318 KB / ~14 MB; Z3 per-query timeout 5 s.
- `tests/benchmarks/` and `benchmarks/error-recovery` measure repair-loop success with a deterministic mock, not compile speed, runtime speed, or LLM token cost. `.github/workflows/benchmarks.yml` exists; results not inspected.
- Evidence of compile-time, runtime speed vs other WASM toolchains, or LLM success rate: none.

## Known limitations

- Agents-only by design: no text syntax, no human review surface, no pretty-printer. JSON ASTs are verbose; human audit relies on `edict_explain` and tooling.
- Contract verification covers linear integer/real arithmetic, booleans, if/let/match; Strings, Arrays and calls to unknown functions are undecidable (`docs/contract-coverage.md`). Non-linear arithmetic may time out.
- Memory default 1 MB and an MVP interpreter; WASM GC/threads/SIMD not used (not verified further).
- Effects are 5 coarse labels; no resource/path-scoped capabilities; imports effect-opaque.
- Spec/implementation drift: unit composition claimed but not implemented; `SECURITY.md` stale; README test count (2675) differs from static count (2346 call sites).
- Release cadence: last release 2026-04-15; ~6 months of unreleased commits at HEAD.
- Tiny community: 13 stars, 2 forks, single maintainer; 167 closed issues (many are maintainer-created task tracking). No external adopters or LLM-eval results documented.
- Self-hosting contract phase is "Partial"; the Z3 path requires native/WASM Z3 (~14 MB in browser).
- The full local suite passed in the second pass (136 files, 2674 passed, 1 skipped); CI run results on GitHub were not inspected.

## Ideas worth adopting for AQELITH

1. Node-id-addressed patch protocol (`replace/delete/insert` over stable ids, deep clone, atomic reject). Pair with content-hash ids rather than author-assigned strings.
2. Structured error contract: every diagnostic carries nodeId, expected/actual, candidates, counterexample, and a pointer to an explain tool. Ship an `errors` catalog as a machine-readable resource.
3. Token-cost features for agents: minimal-schema resource, compact AST form, and an `examples` tool with difficulty tiers.
4. Z3 in a worker with hash-keyed cache and a hard timeout that yields a typed `verification_timeout` rather than a hang. Report "undecidable" explicitly (INFO diagnostics for skipped checks) instead of silent success.
5. Deterministic record/replay of host effects so agent runs are reproducible and debuggable.
6. Effect declaration inspectable before execution plus an `approval` gate that propagates through callers (human-in-the-loop as a type-level property).
7. Publish a coverage table of what the verifier proves/refutes/cannot decide (as `docs/contract-coverage.md` does). Do better: measure it in CI and fail on drift.
8. Avoid their gaps: provide a lossless human-readable projection of the AST (their biggest differentiator-versus-risk), real-path sandbox checks, default-deny network, scrubbed child env, effect checks across module boundaries, and a benchmark with real LLMs rather than a mock repair function.
9. Keep verification tiers portable: a small built-in solver for the common QF-LIA fragment with Z3 as an optional, stronger backend, as Edict does, but gate the weaker solver's soundness with differential tests against Z3.

## Confidence

| Section | Level | Why |
|---|---|---|
| Sources | High | URLs resolved via search/GitHub API; directory mirrors treated as secondary |
| Identity verification | High | Single matching repo; package metadata and layout match the requested feature set |
| Inspected version | High | Commit hash and tag read from the local clone |
| Commands executed | High | First-pass failures recorded as observed (root cause inferred as host disk full / Docker Desktop failure); second-pass build and test results copied from the build log |
| Feature audit | Medium | Source and test files read for each row, and the full suite passed locally in the second pass (not mapped row by row); several rows rest on docs and test file names; `FEATURE_SPEC` is 995 lines and skimmed rather than fully cross-checked |
| Architecture notes | Medium-high | Based on directory layout, README, key source heads; internals of codegen/IR not read line by line |
| Security considerations | Medium | Code-reading hypotheses (symlink escape, default-open HTTP, env inheritance) not tested; UASF trust model not fully audited |
| Performance evidence | Low | No measurements; self-reported numbers only |
| Known limitations | Medium | Mix of documented limits and inferred ones; CI status and unreleased changes not examined |
| Ideas worth adopting | Medium | Opinion grounded in verified features; item 9 and the sandbox hardening items are suggestions, not validated |
