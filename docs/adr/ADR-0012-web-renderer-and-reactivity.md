# ADR-0012: Web renderer, state and events for the first vertical slice

- Status: **Accepted for the M6 vertical slice** (decided by the project owner, who delegated all design choices, 2026-11-06)
- Date: 2026-11-06

## Context

M6 needs a view-tree format, a renderer, reactivity and event handling. ADR-0006 requires the semantic DOM. ADR-0007 limits resumability to declared state and handler ids. The directive forbids a generalized virtual DOM by default and forbids Canvas as the only renderer.

## Decision

1. **View tree** is a closed set of node kinds: `Element` (allowlisted tag), `Text`, `Button` (must have a non-empty label, which is its accessible name), `Bind` (displays one state cell). Two kinds were added by amendment: `Input` (ADR-0013) and `Link` (ADR-0014). No raw HTML node exists, so escaping is the only path.
2. **Server output is plain semantic HTML.** Static HTML stays visible; nothing is hidden with CSS.
3. **State** is a set of named integer cells in this slice. Only declared cells exist. State is serialized as JSON with `<`, `>`, `&`, U+2028 and U+2029 escaped, so it cannot close its script element (the Naze audit's escaping bug).
4. **Events** are declarative operations, not code. Each button gets a handler id (`h1`, `h2`, ...) assigned in document order. The page carries a table `id -> operation` as JSON. The only v0 operation is `add {cell, delta}`. No code is shipped to the browser for events and no `eval` is used.
5. **Reactivity** is per-cell: an operation reports the cell it changed, and only the `Bind` nodes displaying that cell are repainted. No virtual DOM.
6. **Client runtime** is one external script file (no inline script, so a Content-Security-Policy can forbid inline code). It applies operations and updates bound text.

## Alternatives considered

- Compiled JavaScript per handler: rejected for the slice; it needs a JS code generator and a trust decision about shipped code.
- Virtual DOM: rejected by the directive.
- Full hydration: rejected by ADR-0007.

## Consequences

- Only counters-style interactions, one controlled integer input (ADR-0013) and static same-site links between routes (ADR-0014) are expressible in this slice. Client-side navigation, submit flows and data fetching are not part of the view tree and are not claimed.
- Handler ids depend on document order. That is acceptable only because the table is generated per build with the page (ADR-0007).

## Evidence needed

Evidence for this slice:
- Automated: `web/tests/render.rs` (16 tests, including the input of ADR-0013, the title check and the accessibility findings) checks the rendered HTML as strings: escaping, script-element breakout, allowlisted tags, accessible button names, unknown cells, and that only the external runtime is loaded as code.
- Automated: `web/tests/runtime.test.mjs` (6 tests, `node --test web/tests/runtime.test.mjs`, 3 of them for the counter) executes the real `aqelith-runtime.js` against a minimal DOM stub. It checks that a click repaints only the bindings of the changed cell, that the initial paint shows declared state, and that unknown handlers, undeclared cells and unsafe deltas change nothing. The stub is not a browser: it does not test layout, real event bubbling, or the HTML parser. Mutation check: making `paint` repaint every cell fails the per-cell test.
- Automated in headless Chrome: `web/tests/browser/run.mjs` builds the counter, clicks Increment and Decrement with real events, and checks the value and that no script error occurred. The embedded state JSON stays at its initial value, as designed.
- Not done: keyboard and screen-reader testing; deployment-skew experiment.
- Not done: running the browser test in CI, and the deployment-skew experiment. The per-cell repaint was checked in Chrome manually and, since the runtime test was added, also by the stub-based test above.
