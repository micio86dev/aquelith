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
- Cite URL + commit/release + inspection date for every claim about third-party projects.
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
