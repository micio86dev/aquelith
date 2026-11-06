// Executes the real browser runtime (web/runtime/aqelith-runtime.js) against a minimal
// DOM stub, so per-cell repaint, handler dispatch and input commits are tested automatically.
// Run with: node --test web/tests/runtime.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const runtimeSource = readFileSync(join(here, "..", "runtime", "aqelith-runtime.js"), "utf8");

class Element {
  constructor(attrs, text = "") {
    this.attrs = attrs;
    this.text = text;
    this.value = "";
    this.writes = 0;
  }
  getAttribute(name) {
    return Object.prototype.hasOwnProperty.call(this.attrs, name) ? this.attrs[name] : null;
  }
  get textContent() {
    return this.text;
  }
  set textContent(value) {
    this.text = String(value);
    this.writes += 1;
  }
  closest(selector) {
    if (selector === "[data-handler]") return this.attrs["data-handler"] !== undefined ? this : null;
    throw new Error(`unsupported selector in stub: ${selector}`);
  }
}

// Builds a page with the same island and binding shape the Rust renderer emits.
function mountPage({ cells, handlers, binds = [], buttons = [], inputs = [] }) {
  const islands = {
    "aq-state": new Element({ id: "aq-state" }, JSON.stringify(cells)),
    "aq-handlers": new Element({ id: "aq-handlers" }, JSON.stringify(handlers)),
  };
  const bindEls = binds.map((cell) => new Element({ "data-bind": cell }, String(cells[cell])));
  const inputEls = inputs.map(({ cell, handler }) => {
    const el = new Element({ "data-input": cell, "data-handler": handler });
    el.value = String(cells[cell]);
    return el;
  });
  const buttonEls = buttons.map((id) => new Element({ "data-handler": id, type: "button" }, id));
  const selectors = { "[data-bind]": bindEls, "[data-input]": inputEls };
  const listeners = {};
  const document = {
    getElementById: (id) => islands[id] ?? null,
    querySelectorAll: (selector) => {
      if (!(selector in selectors)) throw new Error(`unsupported selector in stub: ${selector}`);
      return selectors[selector];
    },
    addEventListener: (type, fn) => {
      listeners[type] = fn;
    },
  };
  vm.runInNewContext(runtimeSource, { document, Element });
  const click = (button) => listeners.click({ target: button });
  const change = (input, typed) => {
    input.value = typed;
    listeners.change({ target: input });
  };
  return { bindEls, inputEls, buttonEls, click, change };
}

test("clicking a handler repaints only bindings of the changed cell", () => {
  const page = mountPage({
    cells: { count: 0, other: 5 },
    handlers: { h1: { op: "add", cell: "count", delta: 1 } },
    binds: ["count", "other"],
    buttons: ["h1"],
  });
  const [countEl, otherEl] = page.bindEls;
  const writesBefore = { count: countEl.writes, other: otherEl.writes };

  page.click(page.buttonEls[0]);

  assert.equal(countEl.textContent, "1");
  assert.equal(countEl.writes, writesBefore.count + 1, "bound cell is repainted");
  assert.equal(otherEl.writes, writesBefore.other, "unrelated binding is not touched");
  assert.equal(otherEl.textContent, "5");
});

test("initial paint shows the declared state in every binding", () => {
  const page = mountPage({
    cells: { count: 7, other: 2 },
    handlers: {},
    binds: ["count", "other"],
  });
  assert.deepEqual(
    page.bindEls.map((el) => el.textContent),
    ["7", "2"],
  );
});

test("unknown handler ids, undeclared cells and unsafe deltas change nothing", () => {
  const page = mountPage({
    cells: { count: 0 },
    handlers: {
      h1: { op: "add", cell: "missing", delta: 1 },
      h2: { op: "add", cell: "count", delta: 1.5 },
      h3: { op: "add", cell: "count", delta: 9007199254740993 },
      h4: { op: "remove", cell: "count", delta: 1 },
    },
    binds: ["count"],
    buttons: ["h1", "h2", "h3", "h4", "h9"],
  });
  const [countEl] = page.bindEls;
  const writesBefore = countEl.writes;

  for (const button of page.buttonEls) page.click(button);
  page.click(new Element({ "data-handler": "h9" }, "x"));

  assert.equal(countEl.textContent, "0");
  assert.equal(countEl.writes, writesBefore);
});

test("a valid typed integer sets its cell and repaints only that cell's bindings and inputs", () => {
  const page = mountPage({
    cells: { quantity: 1, other: 9 },
    handlers: { h1: { op: "set", cell: "quantity" } },
    binds: ["quantity", "other"],
    inputs: [{ cell: "quantity", handler: "h1" }],
  });
  const [qtyText, otherText] = page.bindEls;
  const otherWrites = otherText.writes;

  page.change(page.inputEls[0], " 42 ");

  assert.equal(qtyText.textContent, "42");
  assert.equal(page.inputEls[0].value, "42", "the input shows the committed value");
  assert.equal(otherText.writes, otherWrites, "unrelated binding is not touched");
});

test("text that is not a safe integer is rejected and the input is restored", () => {
  const typed = ["1.5", "1e3", "abc", "", "--2", "9007199254740993", "12 34"];
  for (const text of typed) {
    const page = mountPage({
      cells: { quantity: 3 },
      handlers: { h1: { op: "set", cell: "quantity" } },
      binds: ["quantity"],
      inputs: [{ cell: "quantity", handler: "h1" }],
    });
    page.change(page.inputEls[0], text);
    assert.equal(page.bindEls[0].textContent, "3", `state changed for ${JSON.stringify(text)}`);
    assert.equal(page.inputEls[0].value, "3", `input not restored for ${JSON.stringify(text)}`);
  }
});

test("a change on an input with an unknown handler or an add operation does nothing", () => {
  const page = mountPage({
    cells: { quantity: 3 },
    handlers: { h1: { op: "add", cell: "quantity", delta: 1 } },
    binds: ["quantity"],
    inputs: [{ cell: "quantity", handler: "h1" }, { cell: "quantity", handler: "h9" }],
  });
  page.change(page.inputEls[0], "100");
  page.change(page.inputEls[1], "100");
  assert.equal(page.bindEls[0].textContent, "3");
});
