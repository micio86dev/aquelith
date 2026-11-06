# Feature: web dev loop, build hardening, and agent guide fixes

Objective: fix what the first agent trial exposed, make the static build small and safe, and give coding agents a live browser preview that reloads on every change.
Branch: feat/web-dev-loop. Delivery strategy: ask-on-risk (default). Test runner: `cargo test` in `web/` and `compiler/`, `node --test web/tests/*.mjs web/tests/browser/landing.mjs`, `gga run`.
No new crates or npm packages: std only.

## Specs

S1. Agent guide fixes (from the trial, 2026-11-06). The guide must give a complete page file example, state that a page header is `page <name> "<route>" {`, that every statement ends with `;`, and that link is `link "<label>" "<route>";`. The build commands must say which directory to run them from. Owner's words, verbatim (Italian), with English gloss: «in base alla prova fatta metti a posto ciò che devi, deve funzionare tutto perfettamente» (fix what the trial exposed; everything must work perfectly).

S2. Static output is minified: HTML and CSS are written without comments and without whitespace between tags or around CSS punctuation. Visible text, headings, links and CSS string literals are unchanged.

S3. Size baseline is recorded before and after the change, raw and gzip, in `docs/performance/SIZE_BASELINE.md`. No numeric budget is set (docs/performance/PERFORMANCE_BUDGETS.md rule). Owner's words, verbatim (Italian), with English gloss: «deve pesare pochissimo, questa è una regola generale per tutti i compilatori, devono fare linting, compressione, sicurezza, test» (it must weigh very little; every compiler must do linting, compression, security and tests).

S4. Security: the static build carries a Content-Security-Policy meta: `default-src 'none'; style-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'`. The dev server sends `Content-Security-Policy` (adds `script-src 'self'`, `connect-src 'self'`, `frame-ancestors 'none'`), `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Cache-Control: no-store`. It serves only GET, answers 405 otherwise, answers 403 to any Host header that is not `127.0.0.1:<port>` or `localhost:<port>`, serves from memory (never from the disk per request), and reads at most 8 KiB of request head.

S5. Hot reload: `dev_site <file.aq> <style.css> [--port N]` builds the site, serves it on 127.0.0.1, and watches the `.aq` and `.css` files. When a file changes, the open page reloads in about one second. When the source does not check, the page shows the diagnostics in a banner and keeps the last good build. When the dev server restarts, the open page reloads. Owner's words, verbatim (Italian), with English gloss: «mentre i coding agent lavorano deve funzionare l'hotreload sui frontend per mostrare le modifiche di UI e funzionali in realtime» (while coding agents work, frontend hot reload must work, to show UI and functional changes in real time).

S6. Tests: unit tests for the minifier (comments, whitespace, strings preserved), tests for the static CSP, integration tests for the dev server (GET 200, 404, 405, 403 on Host, headers, reload event after a change, error event on a broken source), and a node test for the reload client with a fake EventSource. The existing landing browser test keeps passing.

S7. Docs and status: `docs/guides/CONNECT_CODING_AGENTS.md`, `skills/aqelith/SKILL.md` (drift test `compiler/tests/skill.rs` keeps passing), `docs/PROJECT_STATUS.md`, and `web/README` section for the dev loop. Conventional commits, no attribution lines (owner's CLAUDE.md).

Out of scope: new language features, stateful pages in the language (not in v0), a production server, TLS, any deployment.

## Tasks

T1 (S1) Guide and skill: complete example, header syntax, build directory. Route: inline. Risk: none. Status: code checks done (PAR001, PAGE003, PAGE005 verified with the binary); `gga run` not yet passed. Commit: pending.
T2 (S2, S3, S4) Minifier in `web/src/minify.rs`; static build uses it; static CSP meta in `web/src/source.rs`; size baseline doc. Route: inline. Risk: item 2 (the CSP is a security control; checked: no inline style is emitted, test `static_output_has_no_inline_style_for_the_csp_to_block`). Verifier: covered by the single independent verifier at the end of the feature (see T4). Status: web tests 40 passed, landing browser test passed, sizes in docs/performance/SIZE_BASELINE.md; `gga run` not yet passed. Commit: pending.
T3 (S4, S5) Dev server `web/src/dev.rs` and example `web/examples/dev_site.rs`: in-memory snapshot, watcher by polling mtimes, SSE, Host and method guards, header set. Route: inline. Risk: item 2 (security: a local network server). Verifier: one independent verifier at the end of the feature, on the whole diff of T2 to T4, attacking the Host guard, the method guard, the request-head limit, the in-memory lookup and the header set. Status: done. Independent verifier run (2026-11-06): S4 pass with one deviation (duplicate Host gave 400, now 403 as S4 says); S5 partial, with one blocker (a same-mtime rewrite did not reload). Fixed: the watcher compares content, not mtime; duplicate Host gives 403; event streams capped at 64 (503 beyond); a settle loop rebuilds only after the content is still for one period; the watcher's first stamps are taken before the first build. Test-side races fixed too (subscription confirmed before the write; unique scratch names; bytes read past the confirmation are kept). Result: 15 consecutive parallel runs green (60 tests). Real-browser check: edit → reload, error → banner, fix → reload. Commit: pending.
T4 (S5, S6) Reload client `web/runtime/reload.js`, 6 node tests (`web/tests/reload.test.mjs`), Rust integration tests (`web/tests/dev_server.rs`, 17). Route: inline. Risk: item 2 (inherits T3). Verifier: same as T3. Status: done. Restart-triggered reload is covered by the client unit test only, not by an end-to-end test (open gap). Commit: pending.
T5 (S6) Gates: `cargo test`, `node --test`, landing browser test, `gga run`, clippy. Route: inline. Status: compiler 90/90, web 60/60 (15 runs), reload client 6/6, landing browser test passed, clippy clean with `-D warnings`. `gga run` pending for the final unit.
T6 (S7) Docs and PROJECT_STATUS. The guide's "planned" block is replaced by the real usage. Deviation: `web/README` does not exist in this repo; the guide is the single place for the dev loop instead of creating a new README. Status: guide and PROJECT_STATUS done. Commit: pending.

## Log

L1 (owner, 2026-11-06, verbatim, Italian; gloss in English): «si fai tutto tu e poi fammi vedere i risultati di tutto.» (do it all yourself, then show me the results). Earlier: «ok per la proposta di tradeoff, fai tutto per bene e ricorda che mentre i coding agent lavorano deve funzionare l'hotreload sui frontend per mostrare le modifiche di UI e funzionali in realtime» (accepted the tradeoff proposal; do it properly; hot reload must work while agents work). And: «in base alla prova fatta metti a posto ciò che devi, deve funzionare tutto perfettamente» (fix what the trial exposed; everything must work perfectly).
L2: Trial result (scratchpad trial, agent run): check ok; friction in page header syntax, `;`, link order, build directory; agent could not read SKILL.md under the trial rules (my brief), so part of the friction is in the trial setup, not only in the language.
L3: Decision: tradeoff 1 (minify in build) and 3 (security headers) implemented now; tradeoff 2 (budget) recorded as baseline only, no budget.
