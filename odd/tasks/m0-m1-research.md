# m0-m1-research
Objective: complete AQELITH milestones M0 and M1 (environment audit, competitive research). Branch: `docs/m0-m1-research`. Delivery: ask-on-risk, docs only. Test runner: none (docs); `gga run` for review.

## Specs
- S1: "Start with M0 and M1. Research deeply, document accurately, and establish the architecture before committing to implementation."
- S2: M0 deliverable: `docs/reports/ENVIRONMENT_AUDIT.md`, plus `AGENTS.md`.
- S3: M1 deliverables: `docs/strategy/COMPETITIVE_ANALYSIS.md`, `COMPETITIVE_DIFFERENTIATION.md`, `FEATURE_ADOPTION_MATRIX.md`, `REJECTED_IDEAS.md`, and a research report.
- S4: "Do not rely exclusively on landing pages, marketing claims, summaries, or repository descriptions." Classify each feature: implemented and tested / experimental / partial / planned / marketing claim only / not verifiable.
- S5: Mandatory projects: Zerolang, Astra, Naze, Orison, AICore, Edict.
- S6: Branding: `docs/branding/NAMING_RESEARCH.md` and `BRAND_DECISION.md`. "Do not claim that a domain or trademark is available unless the relevant authoritative registry confirms it."
- S7: "Stop only at the explicit brand/publication approval gate or when a genuine external blocker makes further progress impossible."

## Tasks
- T1 (S2) M0 scaffold + audit — inline — done — commit: pending
- T2 (S4, S5) competitor research x6 — delegated (parallel units) — pending
- T3 (S3) reference ecosystems — delegated — pending
- T4 (S6) branding research — delegated — pending
- T5 (S3) synthesis docs + M1 report — inline — pending

## Log
- L1 (2026-11-05): the user pasted the "AQELITH — MASTER BUILD DIRECTIVE" (24 sections). It is too long to copy here; S1–S7 quote its binding lines verbatim. Plan: `~/.claude/plans/pasted-content-id-3271-aqelith-smooth-horizon.md`.
- L2: Environment probed. Correction to plan: rustup 1.29.0 is installed but the active rustc is the Homebrew one (see audit).
