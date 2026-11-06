// Reload client (web-dev-loop, S5, S6): runs runtime/reload.js against a fake EventSource and a
// fake document, in an isolated vm context. Run with: node --test web/tests/reload.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const code = readFileSync(join(here, "..", "runtime", "reload.js"), "utf8");

class FakeEventSource {
  static instances = [];
  constructor(url) {
    this.url = url;
    this.listeners = {};
    this.onerror = null;
    this.onopen = null;
    FakeEventSource.instances.push(this);
  }
  addEventListener(name, fn) {
    (this.listeners[name] ??= []).push(fn);
  }
  emit(name, data) {
    for (const fn of this.listeners[name] ?? []) fn({ data });
  }
}

function makeEnv() {
  const reloads = { count: 0 };
  const body = { children: [], appendChild(n) { this.children.push(n); } };
  const document = {
    body,
    createElement(tag) {
      return { tag, textContent: "", style: {}, attrs: {}, setAttribute(k, v) { this.attrs[k] = v; } };
    },
  };
  const sandbox = {
    EventSource: FakeEventSource,
    document,
    location: { reload() { reloads.count += 1; } },
    JSON,
    String,
  };
  runInNewContext(code, sandbox);
  const es = FakeEventSource.instances.at(-1);
  return { es, reloads, body };
}

test("opens the same-origin event stream", () => {
  FakeEventSource.instances.length = 0;
  const { es } = makeEnv();
  assert.equal(es.url, "/__aq/events");
});

test("a reload event reloads the page", () => {
  FakeEventSource.instances.length = 0;
  const { es, reloads } = makeEnv();
  es.emit("reload", "{}");
  assert.equal(reloads.count, 1);
});

test("a build-error event shows the diagnostic as text, not as markup", () => {
  FakeEventSource.instances.length = 0;
  const { es, reloads, body } = makeEnv();
  es.emit("build-error", JSON.stringify("x.aq:1:2: PAR001: <b>bad</b>"));
  assert.equal(reloads.count, 0, "an error must not reload the page");
  assert.equal(body.children.length, 1);
  const banner = body.children[0];
  assert.equal(banner.attrs.role, "alert");
  assert.equal(banner.textContent, "x.aq:1:2: PAR001: <b>bad</b>");
});

test("a second error replaces the banner instead of adding one", () => {
  FakeEventSource.instances.length = 0;
  const { es, body } = makeEnv();
  es.emit("build-error", JSON.stringify("first"));
  es.emit("build-error", JSON.stringify("second"));
  assert.equal(body.children.length, 1);
  assert.equal(body.children[0].textContent, "second");
});

test("an unreadable error message is reported, not thrown", () => {
  FakeEventSource.instances.length = 0;
  const { es, body } = makeEnv();
  es.emit("build-error", "{not json");
  assert.equal(body.children[0].textContent, "dev server sent an unreadable message");
});

test("after the stream drops and comes back, the page reloads once", () => {
  FakeEventSource.instances.length = 0;
  const { es, reloads } = makeEnv();
  es.onopen(); // first connection: no reload
  assert.equal(reloads.count, 0);
  es.onerror(); // server stopped
  es.onopen(); // server is back
  assert.equal(reloads.count, 1);
});
