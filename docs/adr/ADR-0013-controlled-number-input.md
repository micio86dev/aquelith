# ADR-0013: Controlled number input (first form slice)

- Status: **Accepted for the M6 form slice** (decided by the project owner, who delegated design choices, 2026-11-06)
- Date: 2026-11-06
- Amends: ADR-0012 (adds a closed view node and an operation; the other decisions stand)

## Context

ADR-0012 lists forms as not claimed. M6 still needs a first form that goes through the same path as the counter: declared state, declarative operations, per-cell repaint, and no code shipped to the browser.

## Decision

1. **New view node** `Input { label, cell }`: a number input bound to one declared integer cell. The label is its accessible name and must not be empty. The renderer emits `<label for>` and `<input type="number" step="1">`, and the input gets a handler id like a button.
2. **New operation** `set { cell }`. It is a declarative operation with no code. The runtime reads the typed text on `change`, trims surrounding whitespace, and accepts it only if the rest matches `^-?\d+$` and is a safe integer. Anything else leaves the state unchanged and restores the input to the cell value.
3. **Commit on change, not on each keystroke.** Each keystroke would mean a repaint and a state write; the owner's principle is fewer, explicit state changes.
4. **No submit, no server validation.** There is no form submission in this slice. The value lives in the page state and is not sent anywhere.

## Alternatives considered

- Text inputs with a free-form value: rejected, since the state is integer-only in this slice and a text cell would need its own type rules.
- Per-keystroke `input` events: rejected for the reason in point 3.
- A generic form element with submit: deferred. It needs the submit semantics that the directive places in the application model, which is not built.

## Consequences

- Only integer cells can be edited. Other value types are not part of the view tree.
- The browser sanitizes `type="number"` values before the runtime sees them: spaces are dropped, so `" 42 "` typed through the value setter becomes empty. The runtime still validates every value, so this sanitization is a convenience and not the security boundary.

## Evidence

- Automated: `web/tests/render.rs` (16 tests in total, 4 for the input) covers the label association, the initial value, the `set` entry in the handler table, the non-empty label rule, undeclared cells, and escaping in the label.
- Automated: `web/tests/runtime.test.mjs` (6 tests, `node --test web/tests/runtime.test.mjs`) runs the real runtime against a DOM stub. It covers a valid integer committing only the bound cell, rejection and restoration for 7 invalid texts, and a `change` on an add-operation handler doing nothing. Mutation check: removing the integer format check fails 1 test.
- Automated in headless Chrome: `web/tests/browser/run.mjs` builds `examples/form` with the compiler, opens it in Chrome, and dispatches real `change` and click events. `42` is committed to the text and the input; `1.5` and `1e3` are refused and the input is restored; the button adds one. Mutation check: a runtime that does not restore refused input fails the test. Real keyboard typing and screen readers were not tested.
- Not done: keyboard and screen-reader testing, real typing in a browser test in CI, and a submit flow.
