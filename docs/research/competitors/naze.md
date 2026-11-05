# Competitor audit: Naze

Audit of Naze (declarative "AI-native" UI language, Rust compiler, WASM + Canvas2D runtime) for AQELITH. Evidence from static source/docs reading and web/GitHub queries. The first pass could not build (host disk full); a second pass built the default workspace members and ran their tests (see Commands executed, item 7). Nothing below is a measured result unless stated; the WASM runtime, playground, benchmarks and the CLI itself were not run.

## Sources

- https://www.naze-lang.com/ (pages: `/`, `/language`, `/faq`, `/ecosystem`, `/why`, `/contact`; site itself is a Next.js app, per `/_next/static` references in fetched HTML)
- https://github.com/jasonmamy/naze-lang (MPL-2.0; created 2026-02-02; last push 2026-03-28)
- GitHub API: `repos/jasonmamy/naze-lang` (2 stars, 0 forks, 0 open issues, 0 releases, 0 tags)
- Local clone (untrusted, read-only): `<local-disk>/aqelith-lab/naze@9e3b301`
- Repo docs read: `README.md`, `CLAUDE.md`, `llms.txt`, `docs/LANGUAGE.md`, `docs/PROTOTYPE.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/TOKEN_EFFICIENCY.md`, `docs/AGENT_RUNTIME.md`, `docs/AGENT_RUNTIME_PLAN.md`, `docs/BRAINSTORM.md`

## Inspected version

- Commit `9e3b301867517850e01c324df9aa3c98eb71d702` ("updated for sigma scatter"), 2026-03-27, HEAD of default branch. 39 commits total, first 2026-02-02.
- No tags/releases exist, although `.github/workflows/release.yml` triggers on `v*` tags.
- Workspace version `0.1.0` (`Cargo.toml`). Inspection date: 2026-11-05.
- Single author/driver; README states the codebase "was built through human-AI collaboration using Claude Code" (README.md:5).

## Commands executed

All via host shell unless noted; nothing from the clone was executed on the host.

1. `git clone https://github.com/jasonmamy/naze-lang naze-tmp` then `mv` to `naze@9e3b301` under `<local-disk>/aqelith-lab/` - OK, 1187 files.
2. Read-only host commands on the clone: `ls`, `grep`, `sed`, `wc`, `git log` (clone only; no git in the AQELITH repo).
3. `curl -sL https://www.naze-lang.com/{,faq,language,ecosystem}` - HTTP 200 (`/why-naze` 404; real path is `/why`). `gh api repos/jasonmamy/naze-lang` and `/releases`, `/tags` - results above.
4. `docker run --rm -v <clone>:/src:ro -v <cache>:/cache rust:1-slim sh -c 'apt-get update; apt-get install pkg-config libxkbcommon-dev libwayland-dev libdbus-1-dev libssl-dev git; cargo fetch --locked'` (fetch step, network enabled). **Failed**: the container exited with `no space left on device`; `cargo fetch` never produced files (cache dir stayed 0 B). Cause: host data volume at 100% (`df`: 140 MiB free on `/System<local-disk>`; the Docker VM disk lives on it). The SSD (54 GiB free) was only used for bind mounts, which does not help apt/layer writes.
5. `docker build --load -t aqelith-naze-build` of a slim `rust:1-slim` + pkg-config/libssl-dev/libxkbcommon-dev image (twice) - **failed** with `ERROR: failed to build: EOF` (Docker daemon unhealthy under the same disk pressure).
6. First pass: planned but NOT run (superseded for build and tests by item 7). The 404,788-byte `crates/naze-runtime/pkg/naze_runtime_bg.wasm` is a committed prebuilt artifact (file size only observed via `ls -la`; not rebuilt, not verified to correspond to source).
7. **Second pass (2026-11-05, after Docker was moved to the SSD and restarted; supersedes item 6).** Build and tests ran on a copy of the clone in `aqelith-lab/build/`, with `--network=none --memory=3g`, `CARGO_BUILD_JOBS=2`, after a separate `cargo fetch --locked` with network on. Image: `rust:1-slim` plus `pkg-config` and `libssl-dev` (built with `docker run` + `docker commit`, because `docker build` failed with EOF); toolchain `rustc 1.99.1`. A first build attempt failed only because `openssl-sys` needed `pkg-config`; no project change was made.
    - `cargo build --release --offline --locked` (default workspace members: `nazec`, `naze-parser`, `naze-compiler`, `naze-ir`, `naze-layout`, `naze-renderer`, `naze-native`, `naze-lsp`, `naze-registry`, `naze-discovery`): exit 0, 251 s, with compiler warnings (15 in `naze-discovery` alone, mostly never-constructed types).
    - `cargo test --release --offline --locked` (same members): exit 0, 18 suites, 465 passed, 0 failed, 0 ignored (summed from `test result` lines). This is more than the 440 `#[test]` attributes counted in item 8 (the difference, e.g. doc tests, was not investigated).
    - Not built or run: `naze-runtime` (WASM-only, excluded from default members, so the committed 404,788-byte wasm is still not rebuilt or verified), `naze-playground`, the `nazec` CLI on the examples, SSR/SSG output, and any benchmark.
8. Static counts: `#[test]` occurrences = 440 (parser 156, nazec 128, compiler 56, discovery 38, layout 19, ir 18, renderer 15, registry 10, runtime/native/lsp/playground 0). Rust LOC: nazec 16.9k, runtime 7.4k, compiler 6.6k, parser 5.1k, discovery 4.5k, ir 3.0k, layout 2.3k. 94 top-level `examples/*.naze` (README says 109; 976 `.naze` files total incl. generated/dataset). CI (`.github/workflows/ci.yml`) excludes `naze-runtime`, `naze-renderer`, `naze-playground` from tests and clippy: the WASM/Canvas/a11y code has zero automated tests in CI.

## Feature audit

| Feature | Classification | Evidence |
|---|---|---|
| Compact declarative syntax (`app`, `column`, `rect`, `state`, `on click`) | Implemented and tested | `crates/naze-parser/src/naze.pest`; 156 parser tests; `examples/counter.naze` |
| Tiered grammar (Tier 0-4) as design partition | Partially implemented | Tiers exist only as comments/tags in `naze.pest:1-10`; Tier 4 "future" |
| Per-tier grammar export ("each tier can be exported independently") | Marketing claim only | `grep -i tier` in `crates/nazec/src/cli.rs` and `grammar.rs` returns nothing; no `--tier` flag; README.md "Pay for only what you use" |
| Whole-grammar GBNF/EBNF export (`nazec grammar --format gbnf\|ebnf`) | Implemented and tested (suite passes; this feature's CLI not run) | `crates/nazec/src/grammar.rs:1-12,725`; converts pest source; `--no-test` flag implies a self-test; the `nazec` unit tests passed in the second pass (item 7), the command itself was not executed here |
| "~157 rules, LL(1)-compatible" | Not verifiable | 157 rule lines counted in `naze.pest`; PEG with ordered choice and a catch-all `element` in `statement` (`naze.pest:20`); LL(1) claim only in docs (`TOKEN_EFFICIENCY.md:135`), no checker found |
| Compile-time component inlining | Implemented and tested | `crates/naze-compiler/src/resolve.rs`; CLAUDE.md "Compile Pipeline"; compiler tests 56 |
| Typecheck with error codes, JSON diagnostics (`nazec check --format json`) | Implemented and tested | README tutorial step 6; `crates/naze-compiler/src/typecheck.rs`; `diagnostic.rs` |
| Binary app representation `app_data.bin` | Implemented but experimental | Custom (de)serializer in `crates/naze-ir/src/lib.rs`; no magic/version header found by grep for `MAGIC\|version`; format unversioned |
| Three outputs from one source (full bin ~7KB, manifest ~1KB, headless ~500B) | Planned | Site `/language` and README claim it; `naze-manifest.json` appears only in `docs/*.md`, zero hits in `crates/`; `AGENT_RUNTIME_PLAN.md` Phase A/B are plans; build writes only 4 files (`build.rs`) |
| `nazec context` JSON project summary for agents | Implemented but experimental | `crates/nazec/src/context.rs`; README step 7 |
| Agent-oriented syntax ("one canonical form") | Partially implemented | Canonical-form claim is a design stance; grammar still has many forms (`if`/`match`/`each`/pipelines/`js` interop); no evidence of measured agent benefit |
| Token-efficiency benchmark (Naze 128 tokens vs React 604) | Not verifiable | Single hand-picked dashboard, one tokenizer, recorded in `docs/BRAINSTORM.md:2056`; no script or fixture in repo; `TOKEN_EFFICIENCY.md:553` admits λ,σ,r are "estimates" |
| Single-source UI architecture (UI+state+data in one `.naze`) | Implemented and tested | `examples/*.naze`; state/data/server fn in same file |
| WASM runtime | Implemented but experimental | `crates/naze-runtime/src/lib.rs` (7365 lines, 0 tests); prebuilt 404,788 B wasm committed (README says 395KB; `CLAUDE.md` says embedded via `include_bytes!`) |
| Canvas2D rendering | Implemented but experimental | `crates/naze-renderer/src/lib.rs:127-360` (`fill_text`, `measure_text`, DPR scaling); 15 tests, excluded from CI |
| Custom layout engine (row/column/stack/grid/flex) | Implemented and tested | `crates/naze-layout/src/lib.rs`; 19 tests; native fallback text measure is `len * font_size * 0.6` (`lib.rs:43-45`) |
| Native desktop rendering (winit + tiny-skia + fontdue) | Partially implemented | `crates/naze-native/`; bundled DejaVuSans (`main.rs:269`); README: "Currently Linux only"; no a11y, no IME code found |
| Mobile | Partially implemented | Only `nazec build --target android`: generates a Gradle project hosting the same web build in a WebView (`crates/nazec/src/android_build.rs:61-95`); no iOS; "mobile" claim on site otherwise unsupported; touch scroll handlers exist in runtime (`lib.rs:505-508`) |
| Routing (`page`, dynamic params, guards, History API) | Implemented and tested | `naze.pest` `page_block`/`guard_def`; `serve.rs` tests `find_page_nodes_*` |
| SSR (`nazec serve`) / SSG (`build --static`) | Implemented but experimental | `serve.rs:162-395`; `html_renderer.rs` (semantic-ish HTML, inline CSS); `build.rs:398-445`; "hydration" = serialized state JSON + canvas swap (see below), not DOM hydration |
| Data fetching (REST/WS/SSE), storage, timers | Implemented but experimental | `LANGUAGE.md`; runtime `lib.rs:541-561,3906`; untested in CI (runtime excluded) |
| Server functions + SQL | Implemented but experimental | `crates/nazec/src/server_fns.rs:251-310`; behind optional `database` feature (`nazec/Cargo.toml`); SQLite + Postgres |
| Database models / declarative queries (`model`, `find`, `insert`) | Implemented but experimental | `naze.pest` `model_def`; `codegen.rs:683-740` `compile_find_to_sql` + 2 tests |
| Accessibility: hidden ARIA DOM mirror | Partially implemented | `lib.rs:6536-6800`; details below |
| SEO: meta/OG/JSON-LD in shell, noscript text | Implemented and tested | `crates/nazec/src/seo.rs` (448 lines, unit tests); `build.rs:34-35` |
| LSP, VS Code ext., playground | Implemented but experimental | `crates/naze-lsp` (0 tests), `editors/vscode`, `crates/naze-playground` |
| Package registry | Implemented but experimental | `crates/naze-registry` (10 tests); no deployed public registry found |
| Discovery Network / trust scores | Implemented but experimental | `crates/naze-discovery` (reference server, 38 tests); no live network; `FAQ` "What prevents malicious apps" is design text |
| Naze Browser, agent runtime crate, live app factory | Planned | `docs/NAZE_BROWSER.md`, `AGENT_RUNTIME_PLAN.md`; no crate |
| Fine-tuned small model beating 70B ("local 3-7B matches cloud") | Marketing claim only | `README.md` "Local AI models"; `ai/` has training scripts/plan (`ai/TRAINING_PLAN.md`) but no published model, eval or results |
| "Production-ready" | Marketing claim only (self-contradicted) | README: "not yet suitable for production applications" |
| Pixel-identical rendering on every platform | Marketing claim only | README FAQ; contradicted by README own note that native has font-metric differences |

## Architecture notes

- Pipeline: `.naze` -> pest PEG parse -> resolve (imports, component inlining, themes) -> typecheck -> codegen to `RenderTree` -> custom binary -> `dist/{index.html, naze_runtime.js, naze_runtime_bg.wasm, app_data.bin}` (`CLAUDE.md`, `build.rs`). The runtime is an interpreter of the render tree, not generated code: no per-app WASM is produced.
- Runtime WASM prebuilt blob is embedded in the `nazec` binary via `include_bytes!`; changing runtime requires separate `wasm-pack` step (CLAUDE.md). Supply-chain note: the committed wasm is not reproducibly tied to source in-repo.
- Layout is in Rust and shared between browser and native paths; only the renderer differs (Canvas2D via web-sys vs tiny-skia). Text measurement is renderer-specific (Canvas `measureText` in the browser; fontdue/estimate natively), so "same layout" holds structurally but not metrically.
- Text input uses hidden DOM `<input>`/`<textarea>` overlays (`lib.rs:4988-5110`) for keyboard/IME capture; caret drawn on canvas (`renderer lib.rs:673`).
- SSR/SSG: `html_renderer.rs` emits `div`/`h1`/`a`/`input` HTML with inline CSS into a `.naze-static` container; on WASM start the canvas is shown and the static container gets class `hidden` (`display:none`) (`build.rs:73-100`, template). So the DOM content exists for crawlers and no-JS, then is hidden from users and AT once WASM runs. `serve.rs` additionally embeds `<script type="application/json" id="__naze_state">` for state.
- Standard shell (non-static) is just `<canvas>` + `<noscript>` text (`build.rs:34-35`).
- Agent story: semantic interaction (`execute action ...`) is described in docs; the only shipped machine-facing surfaces are `nazec context`, `nazec grammar`, `nazec parse` (AST JSON), `<link rel="alternate" type="application/naze">` in HTML head, and headless `nazec test`. No runnable agent runtime.
- Docs structure is notable: `llms.txt`, `docs/AGENTS.md` (~970 lines), `naze-dev/SKILL.md` as agent-facing references.

## Accessibility and DOM-bypass evidence

Facts only, each with location.

- Claim: "A hidden DOM overlay mirrors canvas content with ARIA attributes ... This is the same approach Flutter Web uses." (README FAQ; `docs/LANGUAGE.md:3121`).
- Implementation: container `div` with sr-only style (1px, `clip`, `overflow:hidden`) and `role="application"` appended to `body` (`lib.rs:6536-6570`). A separate `aria-live=polite` region announces only the **first** newly appeared text per update (`lib.rs:6620-6630`).
- Rebuild strategy: on every render pass `container.set_inner_html("")` then rebuild all nodes from the layout tree (`lib.rs:6609-6613`; called at `lib.rs:1910`). Consequence observable from code: AT-held references/focus inside the mirror are destroyed on each update.
- No event listeners are attached to mirror elements (no `add_event_listener`/`Closure` in `lib.rs:6536-6900`); `tabindex=0` is set on click-handler nodes (`lib.rs:~6730`) but AT "activate" on a mirror node cannot reach the canvas handler, and DOM focus is not synchronized to the canvas focus ring (canvas focus is runtime-internal Tab handling, `lib.rs:6240-6270`).
- Mirror nodes are not positioned over their canvas counterparts (container is 1px, clipped), so there is no bounding-box geometry for touch-exploration (TalkBack/VoiceOver) or screen magnifiers. Not tested; inferred from the styles above.
- Checkbox state is not mirrored: comment in code "We'd need state here - for now just set the structure" (`lib.rs:~6700`); only `data-bind` set.
- `role="application"` on the container is what the code sets (`lib.rs:6566`); this role changes screen-reader virtual-cursor behavior (general ARIA behavior, not tested here).
- Roles inferred only for heading, link, checkbox, radio, select(listbox), image, scroll(region), clickable row/column as button (`get_a11y_role`, `lib.rs:~6740-6760`); no landmark inference, no tables/lists, no `aria-expanded/selected/checked/current`, no labelled-by relations (`LANGUAGE.md` lists only `role`, `label`, `tab-index`, `id`).
- Compile-time checks: warnings for clickable elements without `role`/`label`, inputs without labels, images without `alt` (`LANGUAGE.md:3136-3139`); `assert no accessibility violations` in test DSL (headless; checks compiler warnings, not a real AT run). Warnings, not errors.
- Browser behaviors given up (derived from architecture; no counter-evidence in repo): native text selection and copy (no selection code found; only a `clipboard.write_text` action at `lib.rs:3623`), Ctrl/Cmd+F find-in-page of live content, browser translation, autofill (inputs are hidden overlays), spellcheck on canvas text, link context menu/middle-click (links are canvas hit-tests), reader mode, forced-colors/high-contrast and user stylesheet overrides, text zoom via browser font size (canvas fonts set in px), RTL/bidi (no `rtl|bidi|dir=` hits in layout/renderer/runtime), `lang` attribute per page (HTML shell has `<html lang="en">` only in static template).
- SEO: mitigation exists via SSG/SSR text + meta + JSON-LD (`seo.rs`, `html_renderer.rs`). Limits: content is hidden after hydration (`display:none` class) which differs from what users see; crawlers executing JS see canvas only (rendered output is not text). Cloaking-risk and parity-drift between `html_renderer.rs` (HTML/CSS approximation) and canvas layout are inherent; there is no test comparing them.
- Interop: `js "expr"` calls through `Reflect` on `window` paths (`lib.rs:3635-3660`), WASM imports; no way to embed arbitrary DOM/web components or iframes found; `overlay` elements are canvas-drawn (`lib.rs:2395`), not top-layer `<dialog>`.
- Naze's own docs partly acknowledge the issue: `PROTOTYPE.md:435-473` plans native a11y APIs (NSAccessibility, UIA, ATK) and AOM "when browser support lands" (planned, not implemented). The native desktop path ships none of it (no a11y hits in `crates/naze-native`).

## Security considerations

Static review only; no exploit was attempted.

- JSON injected into `<script type="application/json">` via `serde_json::to_string` with no `<`/`</script>` escaping (`serve.rs:509-517`, template line 59). serde_json does not escape `<` by default, so state strings containing `</script>` could break out in SSR. Confirmed by reading, not by execution.
- `escape_html` escapes `& < > "` only (`seo.rs:140`), not `'`; check attribute-context usage (`escape_attr` in `html_renderer.rs:479`) before trusting.
- Server functions forward `authorization`, `cookie`, `x-api-key` headers to outgoing fetches (`serve.rs:207-209`); URL `{name}` interpolation substitutes state values verbatim (`server_fns.rs:60-90`): SSRF/parameter injection surface if state is user-controlled.
- SQL goes through numbered placeholders `$N` -> `?N` (parameterized, `server_fns.rs:162-189,291-310`), which is the safer pattern; `create_tables_sqlite` builds DDL via `format!` from model identifiers (`server_fns.rs:238`), identifier-injection only if model names are untrusted (compile-time source).
- `js "..."` interop and `import x from "x.wasm"` run arbitrary JS/WASM in page origin; there is no permission model. Credentials in `localStorage` are acknowledged as readable by any same-origin JS (`NAZE_BROWSER.md:514`).
- Discovery network "trust score" and "prevents malicious apps" are design text; reference server uses optional API key (`--api-key`).
- Dev server and registry are Axum services; no CORS/CSRF/rate limit handling found in `serve.rs` (grep for cors/csrf/rate returned only auth-header forwarding).
- Supply chain: prebuilt WASM blob committed; dependency graph includes `reqwest`, `tokio`, `axum`, `winit`, `rusqlite (bundled)`. `Cargo.lock` is committed. `cargo audit` not run.
- Untrusted-code policy followed: clone only read, no build/run on host.

## Performance evidence

- **No performance measurements by me.** The default workspace builds (251 s release build, see Commands executed item 7), but I did not run any benchmark or the WASM runtime, and the 251 s figure comes from a 4 GiB Docker VM limited to 2 jobs, so it is not a compiler speed claim. All numbers below are the project's own claims or file sizes.
- Committed wasm: 404,788 bytes (observed `ls`). README/site say 395KB; `HISTORY.md` says 406KB -> 374KB at M41 then listed 395KB. Inconsistent; may be gzip/wasm-opt differences. Release profile: `opt-level="z"`, LTO, `panic="abort"` (`Cargo.toml`), i.e. size-optimized, not speed-optimized.
- Claimed app sizes: counter 567 B `app_data.bin`; todo "~7KB full / ~500B headless / ~1KB manifest": headless and manifest not produced by the code (see audit).
- Token benchmark: `BRAINSTORM.md:2056`, one UI, cl100k_base; no repro artifact. `TOKEN_EFFICIENCY.md` σ/λ values are "estimates" (line 553). The README cost table (55K-110K vs 700K-1.8M tokens) is therefore model-based, not measured.
- Runtime architecture note: the a11y DOM is fully cleared and rebuilt each render (`lib.rs:6613`), and layout recomputed per frame except cached for layout-invariant animations (`lib.rs:1838-1845`); no profile data exists in repo. No benchmark directory, no criterion benches, no frame-time numbers.
- Fonts: browser relies on system fonts via `ctx.set_font`; native bundles DejaVuSans.

## Known limitations

- Not production-ready (README); 2 stars, 0 issues, 0 releases, 0 tags, single-commit-author cadence (39 commits, 2026-02 to 2026-03), no external contributors visible. Repo inactive after 2026-03-28 as of inspection.
- Tested surface skewed: runtime, native, LSP, playground have 0 `#[test]`; CI excludes runtime/renderer/playground; 440 tests concentrate in parser/compiler/CLI.
- Native desktop: Linux only; no iOS; Android = WebView wrapper around the web build, i.e. not native rendering.
- Documentation counts drift (109 vs 94 examples; "~100" vs 157 grammar rules in `llms.txt` vs README; 395KB vs 404,788 B).
- Marketing/plans vs code: manifest and headless binary outputs, per-tier export, trained model, Naze Browser, discovery network, production a11y bridge.
- Accessibility, find, selection, translate, RTL, zoom gaps listed above.
- Build reproducibility: runtime wasm is prebuilt and committed.
- Fixed-size 0.6 em text-width estimate in layout crate fallback.

## Ideas worth adopting for AQELITH

1. Lossless project-context export for agents (`nazec context`, `parse --json`, error codes plus `suggested_fix` in JSON diagnostics): adopt machine-readable diagnostics with stable codes.
2. Grammar export for constrained decoding (pest->GBNF/EBNF). Do it properly: per-tier/subset export plus a CI check that generated grammar accepts all examples and rejects fixtures.
3. Agent-facing docs bundle (`llms.txt`, condensed `AGENTS.md`, SKILL file) shipped with the toolchain.
4. Compile-time component inlining plus a single-file-per-component convention to bound the context an agent must read (their "sigma" idea). Treat as a hypothesis and measure it.
5. SSG/SSR emitting real semantic HTML plus meta/JSON-LD from the same tree; keep the DOM visible (do not hide it) to avoid cloaking and a11y loss.
6. Headless test runner using the same language (`.test.naze`, `assert no accessibility violations`) so agents can verify UI without a browser.
7. Build a manifest of state/actions/server functions per app as a first-class artifact (they plan it; implement and version it).
8. Anti-lessons: do not bypass the semantic DOM for general web UI; do not ship claims (tier export, three outputs, pixel parity) ahead of code; version the binary format (magic + version); test the runtime and a11y layers in CI; avoid `role="application"`, full re-creation of the a11y tree per frame, and hidden-then-removed server HTML.

## Confidence

- Sources / Inspected version: high - commit hash, tag/release counts, API results directly observed.
- Commands executed: high - failures recorded verbatim in substance; disk-full cause supported by `df`; second pass results copied from the build logs.
- Feature audit: medium - classifications from source reading and grep; the default workspace members' tests pass (465), but the WASM runtime, playground and CLI behavior were not executed, so "tested" for those means tests exist or are absent, not that the feature works.
- Architecture notes: medium-high - pipeline and crate roles confirmed in source; WASM runtime internals only sampled.
- Accessibility and DOM-bypass evidence: medium - code locations read directly; effects on actual screen readers/browsers not tested (marked as derived where so).
- Security considerations: medium-low - static reading only; the script-tag escaping issue is a code observation, not a reproduced exploit.
- Performance evidence: low - nothing measured; only file sizes and project claims.
- Known limitations: medium-high - mostly derived from observed absence in code (absence may be a grep miss).
- Ideas worth adopting: medium - judgment, grounded in the above.
