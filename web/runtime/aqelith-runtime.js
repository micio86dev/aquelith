// AQELITH web runtime, first vertical slice (ADR-0012).
// Applies declarative handler operations to declared state cells and updates bound text.
// No code is loaded or evaluated from the page data.
"use strict";
(function () {
  const state = JSON.parse(document.getElementById("aq-state").textContent);
  const handlers = JSON.parse(document.getElementById("aq-handlers").textContent);

  // Repaint only the bindings that display `cell`; with no argument, repaint all.
  function paint(cell) {
    for (const el of document.querySelectorAll("[data-bind]")) {
      const name = el.getAttribute("data-bind");
      if (cell !== undefined && name !== cell) continue;
      if (Object.prototype.hasOwnProperty.call(state, name)) {
        el.textContent = String(state[name]);
      }
    }
  }

  // Applies the operation and returns the changed cell, or null if nothing changed.
  function apply(op) {
    if (op.op !== "add") return null;
    if (!Object.prototype.hasOwnProperty.call(state, op.cell)) return null;
    if (!Number.isSafeInteger(op.delta)) return null;
    state[op.cell] = state[op.cell] + op.delta;
    return op.cell;
  }

  document.addEventListener("click", function (event) {
    const button = event.target instanceof Element ? event.target.closest("[data-handler]") : null;
    if (!button) return;
    const op = Object.prototype.hasOwnProperty.call(handlers, button.getAttribute("data-handler"))
      ? handlers[button.getAttribute("data-handler")]
      : null;
    if (!op) return;
    const changed = apply(op);
    if (changed !== null) paint(changed);
  });

  paint();
})();
