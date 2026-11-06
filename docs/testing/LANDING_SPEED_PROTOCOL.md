# Protocol: development speed, AQELITH versus an existing stack

Status: **partly run.** The agent runs (two per arm) are recorded in `docs/reports/LANDING_SPEED_RESULTS.md`. The owner's runs and the third repetition are pending. Written 2026-11-06 for the local experiment. It defines how to measure, so that numbers are reproducible and cannot be read as a claim about the language until they are run.

## 1. Question

For the same landing site (three routes, headings, paragraphs, links), how long does it take an author to reach a build that passes its checks and renders correctly, in AQELITH versus a reference stack?

## 2. Why this is hard to measure

- An agent's speed says little about a person's speed, and the reverse. Both are needed, and they are reported separately.
- A hand-written baseline written by the same person who wrote the compiler is biased. The baseline must be written by someone else, from the spec alone.
- Output size is not development speed. It is reported only as a side metric.

## 3. Shared specification

The spec is the file `web/examples/landing/SPEC.md`, written from the content of `landing.aq` without AQELITH syntax. It lists the routes, the title and heading of each page, the paragraph texts, and the links. Both arms receive the identical spec, copied verbatim.

## 4. Arms

| Arm | Stack | Author |
|---|---|---|
| A | AQELITH, `landing.aq`, `build_site` | Owner, then one agent |
| B | Reference stack chosen before the run (for example plain HTML, or a common static generator) | Owner, then one agent |

Each arm runs twice: once by the owner, once by a fresh agent session with no memory of the other arm. The agent gets the connection guide (`docs/guides/CONNECT_CODING_AGENTS.md`) for arm A and the stack's own documentation for arm B.

## 5. Measures

Recorded for every run:

1. Wall-clock time from receiving the spec to the first build that passes all checks.
2. Number of diagnostics or build errors seen, and how many were fixed by the author.
3. Number of edits to source files, counted with `git diff --stat` or a copy diff.
4. Lines and bytes of the source the author wrote.
5. Whether the rendered result passes the browser check (`web/tests/browser/landing.mjs` for arm A; an equivalent route and link check for arm B).
6. For agents: the count of tool calls and tokens, if the client reports them.

Not recorded as a result: anything that is not measured by this protocol.

## 6. Procedure

1. Freeze the spec and the stack version of arm B. Record the versions and the commit of this repository.
2. Run arm A by the owner. Record the timings in a table, one row per run.
3. Run arm B by the owner, with the same time box and the same spec.
4. Run arm A by a fresh agent, then arm B by a fresh agent. Each agent gets only the spec and its arm's material.
5. Repeat each run three times if the first results are close. Report medians, not the best run.

## 7. Reporting

Results go in `docs/reports/LANDING_SPEED_RESULTS.md`, with: the spec hash, the stack versions, every row of the table, and the raw counts. A conclusion is written only when the difference is larger than the spread across the repeated runs. Otherwise the report says "no measurable difference at this sample size".

## 8. Known limits of the first run

- Arm A has one stylesheet (`style.css`, linked from every page) but no images or lists yet. A real landing page needs those too, so arm A measures a reduced page. The report must say so.
- The browser check confirms that the stylesheet is linked and answers as `text/css`. It does not check the visual result.
- One landing page is one data point. It says nothing about larger applications.
- The browser check for arm A covers navigation and text, not visual quality or performance of the output.
