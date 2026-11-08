# AQELITH — Engineering Contract

Provisional codename. Not cleared for public use (see `docs/branding/`).

## Pillars (architectural requirements, not marketing)
1. Performance-first
2. Security-first
3. Universal compatibility
4. AI-native engineering

## Rules
- Research before implementation; evidence before decisions; measurements before performance claims.
- Never claim AQELITH is faster, safer, or more portable than another language without reproducible evidence.
- Documentation describes implemented behavior only. Mark planned work as planned.
- Competitor audits (`docs/research/competitors/`): cite URL + commit/release + inspection date for every claim about the audited project (master directive §2.3).
- Reference-ecosystem documents (`docs/research/REFERENCE_ECOSYSTEMS.md`): cite URL + access date, plus the document version or revision where the site exposes one; where it does not, say so in the document. They are concept sources, not audited projects (directive §2.2).
- Classify competitor features: implemented-tested / experimental / partial / planned / marketing-only / not-verifiable.
- Source code, comments, docs, commits, and issues are in English.
- Conventional commits. No AI attribution lines in commits.

## Workflow
- GitHub Flow: `main` stays releasable; short-lived `feat/*`, `fix/*`, `docs/*` branches.
- Run `gga run` before considering a unit complete; fix findings, never suppress them.
- Gentle AI (`gentle-ai` 4.3.1) provides ODD/RDD; use only commands shown by its installed help.

## Gates (stop and ask the user)
- Final brand/domain/trademark commitment, or any public repo, package, or release under the name.
- Paid services or purchases.
- Destructive actions on unrelated data.
- Publishing secrets.
- Material change of product direction.

## Isolation
- Third-party code is cloned into `<local-disk>/aqelith-lab/`, never inside this repo.
- Untrusted code is built and run inside Docker (`--network=none` after fetch).
- Exception (owner-approved, 2026-11-08; see `odd/tasks/mobile-android.md`, L35): for the Android A1 host build only, phase 1 of `android/build-in-docker.sh` may run with network and compile the project, because the Android Gradle plugin resolves some of its configurations only when the build tasks run. Build plugins therefore execute with network in that phase. Phase 2 must remain `--network none --offline` and verify every artifact against the Gradle verification metadata. The residual risk is open and recorded in `odd/tasks/mobile-android.md`.
- Exception (owner-approved, 2026-11-07; see `odd/tasks/mobile-android.md`, L17): the Android SDK, its build tools, and the emulator from Google run on the macOS host, outside Docker, because the Android emulator on macOS needs host virtualization (evidence: `emulator -accel-check` exits 0 and reports Hypervisor.Framework, see `odd/tasks/mobile-android.md`, L19). The exception covers only these Google binaries on that host. It does not extend to any other third-party code. Open risk: the trust of these binaries is not verified. `codesign` gives a partial result and `spctl` rejects the emulator (`odd/tasks/mobile-android.md`, L15); the only other check is a SHA-1 match against Google's index (L4). The exception is accepted on that basis, and it is recorded here so the risk is visible.
