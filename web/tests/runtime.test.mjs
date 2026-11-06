// Executes the real browser runtime (web/runtime/aqelith-runtime.js) against a minimal
// DOM stub, so per-cell repaint and handler dispatch are tested automatically.
// Run with: node --test web/tests/
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
function mountPage({ cells, handlers, binds, buttons }) {
  const islands = {
    "aq-state": new Element({ id: "aq-state" }, JSON.stringify(cells)),
    "aq-handlers": new Element({ id: "aq-handlers" }, JSON.stringify(handlers)),
  };
  const bindEls = binds.map((cell) => new Element({ "data-bind": cell }, String(cells[cell])));
  const buttonEls = buttons.map((id) => new Element({ "data-handler": id, type: "button" }, id));
  const listeners = {};
  const document = {
    getElementById: (id) => islands[id] ?? null,
    querySelectorAll: (selector) => {
      if (selector !== "[data-bind]") throw new Error(`unsupported selector in stub: ${selector}`);
      return bindEls;
    },
    addEventListener: (type, fn) => {
      listeners[type] = fn;
    },
  };
  vm.runInNewContext(runtimeSource, { document, Element });
  const click = (button) => listeners.click({ target: button });
  return { bindEls, buttonEls, click };
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
    buttons: [],
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
