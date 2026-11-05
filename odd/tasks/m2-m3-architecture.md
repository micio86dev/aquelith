# m2-m3-architecture
Objective: complete M2 (architecture synthesis: specs, ADRs, risk register, open questions) and M3 (minimal feasibility prototypes that justify or reject architecture choices). Branch: `docs/m0-m1-research` (continues). Delivery: ask-on-risk; docs plus two isolated Rust prototypes with no external dependencies. Test runner: `cargo test` inside each `prototypes/*` crate, on the host (no third-party code involved).

## Specs
- S1: user (2026-11-05): "A me piace aquelith per ora non cambiarlo, procedi con M2 e tutto il resto senza fermarti". Name stays AQELITH. This does not approve public use: the brand gate in `docs/branding/BRAND_DECISION.md` remains PENDING USER APPROVAL.
- S2: M2 deliverables (directive §20): architecture specification, diagrams, ADRs, risk register, open technical questions.
- S3: directive §4 language specs: `docs/language/{LANGUAGE_OVERVIEW,GRAMMAR,TYPE_SYSTEM,ERROR_MODEL,CONCURRENCY_MODEL,MEMORY_MODEL}.md`.
- S4: directive §6 security docs: `docs/security/{THREAT_MODEL,SECURITY_ARCHITECTURE,SECURITY_GUARANTEES,CAPABILITY_MODEL,EFFECT_SYSTEM}.md`. "Do not claim that WASM alone makes an application secure." "Do not claim that frontend checks replace backend authorization."
- S5: directive §7 performance docs: `docs/performance/{PERFORMANCE_MODEL,PERFORMANCE_BUDGETS,BENCHMARK_METHODOLOGY}.md`. "Do not turn aspirational targets into claimed results."
- S6: directive §8 platform docs: `docs/platforms/{PLATFORM_STRATEGY,TARGET_MATRIX,PLATFORM_ABSTRACTION,CONFORMANCE}.md`.
- S7: directive §10 compiler docs: `docs/compiler/{CANONICAL_IR,SEMANTIC_GRAPH,SEMANTIC_PATCH_API,STABLE_IDENTITIES}.md`. "Never trust an agent-provided patch merely because it is syntactically valid JSON."
- S8: directive §9 application/UI model and resumability: `docs/app/{APPLICATION_MODEL,RESUMABILITY}.md`. "Do not promise universal resumability for arbitrary program state."
- S9: directive §14 testing docs: `docs/testing/{TEST_STRATEGY,CONFORMANCE_STRATEGY,SECURITY_TESTING,BENCHMARK_TESTING}.md`; directive §15 `benchmarks/README.md`.
- S10: directive §3 and §8.3 constraints: "Do not allow the Web backend to contaminate the language core." "Do not force WebView as the only mobile or desktop strategy." "Do not use Canvas2D as the only web renderer."
- S11: M3: build minimal isolated prototypes only where an architecture choice needs evidence; discard prototypes that do not justify complexity; deliver a technical feasibility report.
- S12: "Never claim AQELITH is faster, safer, or more portable than another language without reproducible evidence." "Do not mark planned functionality as implemented."
- S13: directive §0.2: do not interrupt for routine decisions; ask only at genuine gates (brand/publication, paid services, destructive actions, secrets, product direction change).

## Tasks
- T1 (S2) ADRs (`docs/adr/`), architecture overview with diagrams, risk register, open questions — inline — done
- T2 (S3) language specs — inline — done — commit: unit 1
- T3 (S4) security specs — inline — done — commit: unit 1
- T4 (S5, S9) performance and testing specs, benchmarks README — inline — done — commit: unit 2
- T5 (S6, S10) platform specs — inline — done — commit: unit 2
- T6 (S7, S8) compiler, application and resumability specs — inline — done — commit: unit 1 (app) and unit 2 (compiler)
- T7 (S2) M2 report — inline — done — commit: unit 3
- T8 (S11) prototype P1: stable identities and validated patches (`prototypes/stable-ids-patch`) — inline — done — 8/8 tests; 2 mutations detected — commit: unit 4
- T9 (S11) prototype P2: capability values vs string tokens (`prototypes/capability-values`) — inline — done — 5/5 tests + compile-fail doctest; root-authority finding (Q23) — commit: unit 4
- T10 (S11) M3 feasibility report — inline — done — commit: unit 4

## Log
- L1 (2026-11-05, verbatim): "A me piace aquelith per ora non cambiarlo, procedi con M2 e tutto il resto senza fermarti"
- L2: Brand decision not changed by L1. Gate remains open.
- L4 (2026-11-05, verbatim): "Puoi fare spazio su disco eliminando cache mode,npm e immagini Docker inutilizzate senza problema se serve". Done: `npm cache clean --force` (711 MB to 44 MB), `pnpm store prune` (619 MB), `docker container prune` (344 MB), `docker image prune -a` (1.33 GB; this removed ALL images, including the lab images `aic-lab:rust*`, `zero-lab:x64`, `naze-lab:rust`, `node:22-bookworm`, `rust:1-slim`, which must be rebuilt before the next Docker build). Internal disk free: 4.3 GiB to 6.8 GiB. NOT touched: `~/Library/Caches` (5.2 GB), `~/.gradle` (1.4 GB), `~/.cargo/registry` (335 MB), and the old `Docker.raw` (51 GB); not named, so not removed.
- L3: No delegation planned for M2/M3. Writing is docs-only and the parent holds the decisions; prototypes are small and inline (Write rule: one understood change per prototype). Each docs unit is one commit under 400 authored lines of prose by planning heuristic; GGA review per commit.
