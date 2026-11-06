// AQELITH web runtime (ADR-0012, ADR-0013).
// Applies declarative handler operations to declared state cells and updates bound text and inputs.
// No code is loaded or evaluated from the page data.
"use strict";
(function () {
  const state = JSON.parse(document.getElementById("aq-state").textContent);
  const handlers = JSON.parse(document.getElementById("aq-handlers").textContent);

  function has(obj, key) {
    return Object.prototype.hasOwnProperty.call(obj, key);
  }

  // Repaint the bindings and inputs that display `cell`; with no argument, repaint all.
  function paint(cell) {
    for (const el of document.querySelectorAll("[data-bind]")) {
      const name = el.getAttribute("data-bind");
      if (cell !== undefined && name !== cell) continue;
      if (has(state, name)) el.textContent = String(state[name]);
    }
    for (const el of document.querySelectorAll("[data-input]")) {
      const name = el.getAttribute("data-input");
      if (cell !== undefined && name !== cell) continue;
      if (has(state, name)) el.value = String(state[name]);
    }
  }

  // Integer text only: no spaces, decimals or exponents. Returns null when the text is not a safe integer.
  function parseInteger(text) {
    const trimmed = String(text).trim();
    if (!/^-?\d+$/.test(trimmed)) return null;
    const n = Number(trimmed);
    return Number.isSafeInteger(n) ? n : null;
  }

  // Applies the operation and returns the changed cell, or null if nothing changed.
  function apply(op, value) {
    if (!op || typeof op !== "object" || !has(state, op.cell)) return null;
    if (op.op === "add") {
      if (!Number.isSafeInteger(op.delta)) return null;
      state[op.cell] = state[op.cell] + op.delta;
      return op.cell;
    }
    if (op.op === "set") {
      if (!Number.isSafeInteger(value)) return null;
      state[op.cell] = value;
      return op.cell;
    }
    return null;
  }

  function handlerOf(element) {
    const id = element.getAttribute("data-handler");
    return id !== null && has(handlers, id) ? handlers[id] : null;
  }

  // Buttons carry "add" operations.
  document.addEventListener("click", function (event) {
    const button = event.target instanceof Element ? event.target.closest("[data-handler]") : null;
    if (!button) return;
    const op = handlerOf(button);
    if (!op || op.op !== "add") return;
    const changed = apply(op);
    if (changed !== null) paint(changed);
  });

  // Number inputs carry "set" operations and commit on change. Invalid text restores the cell's value.
  document.addEventListener("change", function (event) {
    const input = event.target instanceof Element ? event.target.closest("[data-handler]") : null;
    if (!input) return;
    const op = handlerOf(input);
    if (!op || op.op !== "set") return;
    const changed = apply(op, parseInteger(input.value));
    paint(changed !== null ? changed : op.cell);
  });

  paint();
})();
