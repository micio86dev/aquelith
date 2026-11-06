// Browser test (M6): builds the example pages with the compiler, opens them in headless Chrome
// and checks the runtime with real DOM events. The checking script is injected only into a copy
// of the page in a temporary directory; the product page keeps no inline code.
// Run with: node --test web/tests/browser/run.mjs   (CHROME_BIN overrides the Chrome path)
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webDir = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const chrome = process.env.CHROME_BIN ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const skip = existsSync(chrome) ? false : `no Chrome at ${chrome}`;

function buildExample(example) {
  const dir = mkdtempSync(join(tmpdir(), `aq-${example}-`));
  execFileSync("cargo", ["run", "-q", "--example", example, "--", dir], { cwd: webDir, stdio: "pipe" });
  return dir;
}

// Runs `script` after the page has loaded (so the runtime has run) and returns the recorded checks.
function runInChrome(dir, script) {
  const html = readFileSync(join(dir, "index.html"), "utf8");
  const page = html.replace(
    "</body>",
    `<pre id="aq-result"></pre>\n<script>\nwindow.__aqErrors = [];\nwindow.addEventListener("error", (e) => window.__aqErrors.push(String(e.message)));\n${script}\n</script>\n</body>`,
  );
  const testPath = join(dir, "test.html");
  writeFileSync(testPath, page);
  const dom = execFileSync(
    chrome,
    ["--headless=new", "--disable-gpu", "--no-sandbox", "--dump-dom", `file://${testPath}`],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], timeout: 60000 },
  );
  const match = dom.match(/<pre id="aq-result">([\s\S]*?)<\/pre>/);
  assert.ok(match, "the result element is present after load");
  const text = match[1].replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");
  return JSON.parse(text);
}

const common = `
  const checks = [];
  const check = (name, ok) => checks.push({ name, ok: Boolean(ok) });
  const text = () => document.querySelector("[data-bind]").textContent;
  const button = (label) => Array.from(document.querySelectorAll("button")).find((b) => b.textContent === label);
  const finish = () => {
    checks.push({ name: "no script errors", ok: window.__aqErrors.length === 0 });
    document.getElementById("aq-result").textContent = JSON.stringify(checks);
  };
`;

test("counter: buttons change the value and the page shows it", { skip }, () => {
  const dir = buildExample("build_counter");
  const checks = runInChrome(
    dir,
    `window.addEventListener("load", () => {
      ${common}
      check("starts at 0", text() === "0");
      button("Increment").click();
      button("Increment").click();
      check("two increments give 2", text() === "2");
      button("Decrement").click();
      check("one decrement gives 1", text() === "1");
      finish();
    });`,
  );
  for (const c of checks) assert.ok(c.ok, `failed: ${c.name}`);
});

test("form: a valid integer is committed, invalid text is refused and restored", { skip }, () => {
  const dir = buildExample("build_form");
  const checks = runInChrome(
    dir,
    `window.addEventListener("load", () => {
      ${common}
      const input = document.querySelector("input[data-input]");
      const commit = (typed) => { input.value = typed; input.dispatchEvent(new Event("change", { bubbles: true })); };
      check("starts at 1", text() === "1" && input.value === "1");
      commit("42");
      check("42 is committed to the text and the input", text() === "42" && input.value === "42");
      commit("1.5");
      check("1.5 is refused and the input is restored", text() === "42" && input.value === "42");
      commit("1e3");
      check("1e3 is refused", text() === "42" && input.value === "42");
      button("Add one").click();
      check("the button adds one and the input follows", text() === "43" && input.value === "43");
      finish();
    });`,
  );
  for (const c of checks) assert.ok(c.ok, `failed: ${c.name}`);
});

test("the product page keeps no inline code: only JSON islands and the external runtime", () => {
  for (const example of ["build_counter", "build_form"]) {
    const html = readFileSync(join(buildExample(example), "index.html"), "utf8");
    const scripts = [...html.matchAll(/<script([^>]*)>/g)].map((m) => m[1]);
    for (const attrs of scripts) {
      assert.ok(/type="application\/json"/.test(attrs) || /src="aqelith-runtime\.js"/.test(attrs), attrs);
    }
  }
});
