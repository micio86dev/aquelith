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
- The plain reference comparison is one run per arm; see the section below.

## Reference arm: plain HTML, written by a fresh agent

The same spec was given to a second fresh agent, which wrote plain HTML and CSS with no framework. It did not know the project. The build step is a copy, because plain files need no build.

| Measure | AQELITH | Plain HTML |
|---|---|---|
| Authoring tool calls | 6 | 7 |
| Authoring time (headless run) | 18.5 s | 18.4 s |
| Spec terms found | 10 of 10 | 10 of 10 |
| Pages | 4 | 4 |
| HTML raw / gzip, four pages | 2031 / 1354 | 1911 / 1229 |
| Stylesheet raw / gzip | none | 522 / 321 |
| Total raw / gzip | 2031 / 1354 | 2433 / 1550 |
| "Build" time, median of 5 runs, ms | 24 (AQELITH build) | 29 (copy only) |

How to read it:

- The HTML alone is about 6% smaller in the plain version, but the plain version also ships a stylesheet, which AQELITH's trial did not. The totals are not comparable as they stand.
- The AQELITH HTML carries the CSP meta element (about 130 bytes raw per page). Without it the AQELITH HTML would be smaller; I did not measure that version.
- The times are not the same operation. AQELITH's number is a compile; the plain number is a copy. Only the authoring times are comparable, and each is a sample of one.
- The plain agent left out the `site.aq` and `dist/` deliverables, because the spec does not name a build step for plain files. That is a flaw in the spec, not a result.

## Six sessions: three per arm, same spec and harness

Date: 2026-11-06. Each run was a fresh headless `claude -p` session in an empty folder, with the same spec (now asking for one stylesheet) and a per-arm `CLAUDE.md`. Runs were sequential. Raw logs stay outside the repository.

| Arm | Run | Wall time (ms) | Tool calls | MCP calls | Terms found (of 10) | Broken links | HTML raw / gzip | CSS raw / gzip |
|---|---|---|---|---|---|---|---|---|
| AQELITH | 1 | 31574 | 8 | 0 | 10 | 0 | 2195 / 1390 | 165 / 146 |
| AQELITH | 2 | 31392 | 8 | 0 | 10 | 0 | 2195 / 1390 | 284 / 220 |
| AQELITH | 3 | 38767 | 10 | 0 | 10 | 0 | 2195 / 1390 | 176 / 158 |
| Plain HTML | 1 | 28942 | 7 | 0 | 10 | 0 | 2043 / 1223 | 848 / 441 |
| Plain HTML | 2 | 40934 | 11 | 0 | 10 | 0 | 2027 / 1212 | 843 / 419 |
| Plain HTML | 3 | 52184 | 14 | 0 | 10 | 0 | 2043 / 1223 | 760 / 405 |

Medians: AQELITH wall time 31.6 s, 8 tool calls. Plain HTML 40.9 s, 11 tool calls.

What the numbers support, and what they do not:

- **No measurable difference in authoring time at this size.** The AQELITH range is 31.4 to 38.8 s; the plain range is 28.9 to 52.2 s. The ranges overlap, so the protocol's rule says: no conclusion.
- **Both arms passed the content check** in all six runs.
- **The AQELITH agents did not use the MCP tools.** All three called the `aqelith` command-line binary through Bash, because the `CLAUDE.md` gave its path. The MCP server was connected and available. The MCP route was therefore not exercised by these runs, and the guide should say so.
- **The plain-HTML agents used a memory tool** from the user's global configuration in one run. That is not part of the plain arm's design, and it may have added calls.
- **Output sizes differ in the stylesheet, not in the HTML.** The AQELITH build minifies its CSS, so its stylesheet is about a quarter of the plain one. The HTML is larger on the AQELITH side because of the CSP meta element (about 130 bytes per page).
- Totals are not a speed claim. Three runs per arm is a small sample.

## Three sessions with the MCP route

The `CLAUDE.md` now names the MCP tools for validation and formatting, and forbids the binary for validation. Same spec and harness.

| Run | Wall time (ms) | Tool calls | MCP calls | Binary via Bash | Terms (of 10) | Broken links | HTML gzip | CSS gzip |
|---|---|---|---|---|---|---|---|---|
| 1 | 37366 | 9 | 2 | 0 | 10 | 0 | 1390 | 186 |
| 2 | 37194 | 10 | 2 | 0 | 10 | 0 | 1390 | 141 |
| 3 | 32607 | 9 | 2 | 0 | 10 | 0 | 1390 | 157 |

The change worked: each run called `aqelith_check` and `aqelith_format` through MCP, and none used the binary. Median wall time is 37.2 s, against 31.6 s for the binary route and 40.9 s for plain HTML. The ranges overlap across all three arms, so again: no measurable difference at this sample size.

## Next step

Repeat the MCP route: a run whose `CLAUDE.md` tells the agent to call the `aqelith_check` tool, to see whether it uses the server rather than the binary. Then decide whether the gap in authoring time is larger than the spread between runs, with more runs.
