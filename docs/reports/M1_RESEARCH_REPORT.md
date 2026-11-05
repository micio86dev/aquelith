# M0 + M1 Report

Date: 2026-11-05. Scope: M0 (environment and repository) and M1 (competitive research). No language, compiler or runtime code exists yet.

## 1. What was completed

- **M0:** repository initialised (`main`, work on `docs/m0-m1-research`), `AGENTS.md`, `README.md`, `docs/PROJECT_STATUS.md`, `docs/reports/ENVIRONMENT_AUDIT.md`, GGA configured.
- **M1:** six competitor audits (Zerolang, Astra, Naze, Orison, AICore, Edict), a reference-ecosystems document, a naming/branding research report, and the strategy set: competitive analysis, differentiation hypotheses, feature adoption matrix, rejected ideas.
- Not completed: the Zerolang test suite on any platform; Astra's compiler could not be inspected (source private); three AICore test failures are unexplained. See section 8.

## 2. Important decisions

- Local git only; no GitHub remote until the brand gate clears.
- Third-party code lives in `<local-disk>/aqelith-lab/`, outside the repository, and runs in Docker. Policy (`AGENTS.md`): fetch with network, run code with `--network=none`.
- `AGENTS.md` citation rule split: competitor audits need URL + commit/release + date; reference documents need URL + access date + version where exposed (the directive requires commits for §2.3 audits, not for §2.2 references).
- Docker's data folder moved to the SSD (user-approved) after the internal disk filled; the old 51 GiB `Docker.raw` was left in place for the user to delete.
- Strategy outputs are hypotheses, not commitments; M2 and M3 must produce evidence.

## 3. Files created

`AGENTS.md`, `README.md`, `docs/PROJECT_STATUS.md`, `docs/reports/{ENVIRONMENT_AUDIT,M1_RESEARCH_REPORT}.md`, `docs/research/competitors/{zerolang,astra,naze,orison,aicore,edict}.md`, `docs/research/REFERENCE_ECOSYSTEMS.md`, `docs/strategy/{COMPETITIVE_ANALYSIS,COMPETITIVE_DIFFERENTIATION,FEATURE_ADOPTION_MATRIX,REJECTED_IDEAS}.md`, `docs/branding/{NAMING_RESEARCH,BRAND_DECISION}.md`, `odd/tasks/m0-m1-research.md`, `.gga`.

## 4. Tests executed and results (third-party projects, local, Docker)

| Project | Result |
|---|---|
| Orison | build ok (35 s); 1097 tests passed, 0 failed |
| Naze | default members build ok (251 s); 465 passed, 0 failed; WASM runtime and playground not built |
| Edict | build ok; 2674 passed, 1 skipped (README claims 2675) |
| AICore | build ok (119 s); 1342 passed, 5 failed, 2 ignored. 1 environmental (missing `procps`), 1 inferred (root), 3 unexplained (UDP/DNS under no-network, TLS `free(): invalid pointer`, selfhost driver) |
| Zerolang | official `make` build ok (103 s); native test script not completed: AArch64 host fails a conformance fixture (BLD004); x86_64 (emulated) stops silently after the target matrix |
| Astra | no build possible; only a public WASM build was exercised |

## 5. GGA results

- M0 commit: failed once (status doc claimed a commit before it existed), fixed, passed.
- Research commit: failed three times, passed on the fourth attempt. Findings were valid and were fixed in the documents, not suppressed: stale status wording; first-pass wording in the Edict audit contradicting the second pass; non-standard classification labels; an over-broad citation rule; and two runs of untrusted code with the network enabled (Edict install scripts, AICore and Zerolang reruns), which were redone with `--network=none`.
- Branding commit: passed first time.
- The strategy set and this report are reviewed at their own commit (see git log).

## 6. Performance measurements

None that are comparable. Build and test times above come from one M1 laptop with 8 GiB RAM, a 4 GiB Docker VM limited to 2 jobs, partly under emulation. They are observations, not speed claims. No AQELITH performance claim exists.

## 7. Security findings (observations about third-party projects; nothing exploited)

- Naze: hydration JSON is written into a `<script>` tag without escaping `</script>` (found by reading the code).
- Zerolang: `patch` accepts type-invalid literals while `check` prints `ok`; SIGSEGV on deeply nested input reproduced (issue #435).
- Edict: lexical-only file sandbox, HTTP open unless allow-listed, child process inherits the environment.
- Orison and Astra: "capability" mechanisms are strings or an effect lint, not enforcement.
- Process: the first Edict install ran project install scripts with the network on, and two later reruns did too; all were redone compliantly and the deviations are recorded in the audits.

## 8. Known limitations

- Astra: compiler and tests private; every Astra performance and feature claim beyond the public WASM build is NV.
- Zerolang test suite not completed; its published release binary was not compared.
- AICore: three failing tests unexplained.
- Orison identity rests on self-description (1 star, 9 commits in 2 days).
- Reference-ecosystem document records no per-source version.
- Every audit is a single-machine, single-day snapshot.
- Trademark status is unknown: EUIPO, WIPO, USPTO and UIBM could not be queried automatically.

## 9. Remaining risks

- **Brand:** AQELITH has near look-alikes in use (Aqualith, Aquilith, Aquolith, Aquelit) and related "Aqu-" software marks exist; legal clearance needs the manual registry steps in `docs/branding/NAMING_RESEARCH.md`.
- Evidence depth differs by project, so the matrix is not a ranking.
- Host: the internal disk had 2–3 GiB free; Docker needs the SSD data folder to stay in place.

## 10. Next milestone

M2 (architecture synthesis). Inputs: adoption matrix rows 5, 8, 9, 11, 15, 21 and differentiation hypotheses D1–D5. Open questions M2 must resolve: single source authority for text vs graph; capability-value design; resumability and deployment skew; stack validation (Rust is the starting hypothesis, the toolchain is not pinned).

## Pending user decisions

- **Brand gate:** `docs/branding/BRAND_DECISION.md` is PENDING USER APPROVAL. No public repository, package, release or domain purchase is allowed until then.
- Old `Docker.raw` (51 GiB) on the internal disk: delete when you have confirmed you don't need its images and volumes.
