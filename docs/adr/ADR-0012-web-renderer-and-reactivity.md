# ADR-0012: Web renderer, state and events for the first vertical slice

- Status: **Accepted for the M6 vertical slice** (decided by the project owner, who delegated all design choices, 2026-11-06)
- Date: 2026-11-06

## Context

M6 needs a view-tree format, a renderer, reactivity and event handling. ADR-0006 requires the semantic DOM. ADR-0007 limits resumability to declared state and handler ids. The directive forbids a generalized virtual DOM by default and forbids Canvas as the only renderer.

## Decision

1. **View tree** is a closed set of node kinds: `Element` (allowlisted tag), `Text`, `Button` (must have a non-empty label, which is its accessible name), `Bind` (displays one state cell). No raw HTML node exists, so escaping is the only path.
2. **Server output is plain semantic HTML.** Static HTML stays visible; nothing is hidden with CSS.
3. **State** is a set of named integer cells in this slice. Only declared cells exist. State is serialized as JSON with `<`, `>`, `&`, U+2028 and U+2029 escaped, so it cannot close its script element (the Naze audit's escaping bug).
4. **Events** are declarative operations, not code. Each button gets a handler id (`h1`, `h2`, ...) assigned in document order. The page carries a table `id -> operation` as JSON. The only v0 operation is `add {cell, delta}`. No code is shipped to the browser for events and no `eval` is used.
5. **Reactivity** in this slice repaints every `Bind` node after each event (`paint()` in the runtime). Per-cell repainting is the intended next step, not implemented. No virtual DOM.
6. **Client runtime** is one external script file (no inline script, so a Content-Security-Policy can forbid inline code). It applies operations and updates bound text.

## Alternatives considered

- Compiled JavaScript per handler: rejected for the slice; it needs a JS code generator and a trust decision about shipped code.
- Virtual DOM: rejected by the directive.
- Full hydration: rejected by ADR-0007.

## Consequences

- Only counters-style interactions are expressible in this slice. Forms, routing and data fetching are not yet part of the view tree and are not claimed.
- Handler ids depend on document order. That is acceptable only because the table is generated per build with the page (ADR-0007).

## Evidence needed

Evidence for this slice:
- Automated: `web/tests/render.rs` (10 tests) checks the rendered HTML as strings: escaping, script-element breakout, allowlisted tags, accessible button names, unknown cells, and that only the external runtime is loaded as code. No automated test executes the JavaScript runtime.
- Manual, not automated: on 2026-11-06 the counter was served locally and clicked in Chrome; the value went 0 to 1 to 2 to 1 with both buttons. The embedded state JSON stayed at its initial value, as designed.
- Not done: a browser test in CI, the accessibility checks, and the deployment-skew experiment.
