# Mini site trial: a fresh Claude Code session writes a four-page site

Date: 2026-11-06. Status: one run, one author (a fresh headless Claude Code session). This is a sample of one; it says nothing about the spread of results.

## Question

Can a fresh coding agent, given only the spec and the connection guide, produce a static multi-page site in AQELITH that passes its checks and builds, and what does the build cost?

## Setup

- Spec: four pages (`/`, `/features`, `/pricing`, `/contact`), two of them with level-two headings, with links between pages. The spec text is kept outside the repository (scratch folder).
- Agent: `claude -p` (non-interactive), in an empty folder with a `CLAUDE.md` taken from `docs/guides/CONNECT_CODING_AGENTS.md` section 3. Tools allowed: file read and write, bash, and the five `aqelith` MCP tools. The MCP server and the skill were the user-level installation from the guide.
- Build: `build_site` from `web/`, release profile.

## Results

| Measure | Value |
|---|---|
| Agent tool calls | 6 (Read 2, Write 1, `aqelith_check` 1, Bash 2) |
| Agent wall time (headless run) | 18.5 s |
| Diagnostics seen / fixed | 0 / 0 (first `check` passed) |
| Build passed | yes, 4 pages written |
| Spec terms found in the output | 10 of 10 checked by grep |
| Internal links that resolve | all |
| Output size, four pages, raw / gzip | 2031 / 1354 bytes |
| Build time, five warm runs, ms | 35, 24, 23, 24, 24 (median 24) |

The agent reported 0 diagnostics and a passing build. I checked both again myself: `check` returned `ok`, and the build command wrote the four pages.

## What this does not show

- No comparison with another stack. A baseline written by hand from the same spec would be needed before any statement about relative speed. None was made.
- The build time covers the AQELITH build only, not the agent's authoring time as a human would experience it.
- Each page carries a Content-Security-Policy meta element (about 130 bytes raw per page). Those bytes are included in the sizes above.
- Behaviour in a browser was not checked for this site.

## Next step

Write the same four-page site by hand, from the same spec, as the reference arm of `docs/testing/LANDING_SPEED_PROTOCOL.md`, and repeat both arms three times.
