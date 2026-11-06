// AQELITH web runtime, first vertical slice (ADR-0012).
// Applies declarative handler operations to declared state cells and updates bound text.
// No code is loaded or evaluated from the page data.
"use strict";
(function () {
  const state = JSON.parse(document.getElementById("aq-state").textContent);
  const handlers = JSON.parse(document.getElementById("aq-handlers").textContent);

  function paint() {
    for (const el of document.querySelectorAll("[data-bind]")) {
      const name = el.getAttribute("data-bind");
      if (Object.prototype.hasOwnProperty.call(state, name)) {
        el.textContent = String(state[name]);
      }
    }
  }

  function apply(op) {
    if (op.op !== "add") return;
    if (!Object.prototype.hasOwnProperty.call(state, op.cell)) return;
    if (!Number.isSafeInteger(op.delta)) return;
    state[op.cell] = state[op.cell] + op.delta;
  }

  document.addEventListener("click", function (event) {
    const button = event.target instanceof Element ? event.target.closest("[data-handler]") : null;
    if (!button) return;
    const op = Object.prototype.hasOwnProperty.call(handlers, button.getAttribute("data-handler"))
      ? handlers[button.getAttribute("data-handler")]
      : null;
    if (!op) return;
    apply(op);
    paint();
  });

  paint();
})();
