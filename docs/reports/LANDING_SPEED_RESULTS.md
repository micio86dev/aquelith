# Landing speed results: AQELITH (arm A) versus plain HTML (arm B)

Protocol: `docs/testing/LANDING_SPEED_PROTOCOL.md`. Run on 2026-11-06 on one machine, locally, with no network use by the sites.

## Conditions

- Spec: `web/examples/landing/SPEC.md`, SHA-256 `5755f3358895728f0924d9ba55d3f367826573a51d000ef55ee30d012d44f1e7` (the copy given to every agent).
- Arm A: AQELITH compiler and web backend from this branch (commits `fa1fa77`, `12a2964`). The runs used the working tree before the non-ASCII fix in `lexer.rs`; the spec is ASCII, so the fix does not change these runs. Material given to the agent: `skills/aqelith/SKILL.md`, `docs/language/GRAMMAR.md`, and the connection guide. The agent was told not to open `web/examples/`.
- Arm B: hand-written HTML5 and one CSS file, no framework, no build tool, no JavaScript. Reference choice: it needs no install and no version pin, and it is the output that arm A produces. Material given to the agent: the spec only.
- Each run is one fresh general-purpose agent with no memory of the other arm. Timestamps are written by the agent itself with `date +%s`, so each figure includes the agent's own thinking time between calls. These are agent runs, not the owner's runs. The owner's own arm is not run yet.
- Checker: each site was served on loopback and checked in headless Chrome against the spec (title, h1, h2, paragraph, links, one stylesheet served as `text/css`, no script). Script: a scratch checker, not committed.

## Results

| Run | Arm | Seconds, start to first pass | Tool calls | Build diagnostics | Edits after first write | Browser check vs spec |
|---|---|---|---|---|---|---|
| 1 | A | 17 | 9 | 0 | 0 | PASS |
| 2 | A | 18 | 9 | 0 | 0 | PASS |
| 1 | B | 19 | 9 | not applicable | not applicable | PASS |
| 2 | B | 12 | 8 | not applicable | not applicable | PASS |

Source written by the author (bytes, lines, from the files each agent wrote):

| Run | Arm | Authored files | Lines | Bytes | Generated output |
|---|---|---|---|---|---|
| 1 | A | `landing.aq` 24 lines, 687 B; `style.css` 23 lines, 291 B | 47 | 978 | HTML pages and style, 59 lines |
| 2 | A | same kind | 47 | 983 | same kind |
| 1 | B | HTML pages and `style.css` | 108 | 2089 | the same files (no generation step) |
| 2 | B | HTML pages and `style.css` | 109 | 2124 | the same files |

## Reading

- Protocol section 7: a conclusion is written only when the difference is larger than the spread across repeated runs. Arm A took 17 to 18 seconds; arm B took 12 to 19 seconds. The ranges overlap.
- **Result: no measurable difference at this sample size.** Two runs per arm are not enough to rank the two. The protocol asks for three; the third round is not run, because the overlap already prevents a conclusion in either direction.
- The source size of arm A is about half of arm B's, but arm A's source compiles to the HTML. Size is a side metric and is not development speed.
- Arm A's agents passed the first build with no diagnostics in both runs. That shows the skill and grammar were enough for this small slice. It says nothing about larger programs.

## Limits

- A landing site of three pages with one stylesheet. No images, no lists, no client-side behaviour. Arm A is a reduced page, as the protocol says.
- Two samples per arm, measured on agents, not people. The owner's manual runs are still needed for the human arm.
- Arm B is a language the agent already knows well; arm A is a language it learned from the docs in the same run. The two conditions differ in prior knowledge, and this comparison does not separate that effect from the language effect.
- The browser check verifies the output against the spec. It does not check the visual result.

## Not claimed

Nothing here shows that AQELITH is faster, safer or more portable than HTML. AGENTS.md requires reproducible measurements for such claims; this run is one small data point.
