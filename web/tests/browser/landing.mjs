// Landing site test (ADR-0015): builds examples/landing/landing.aq with the compiler, serves the
// output over loopback HTTP and checks every route in headless Chrome, plus that every link answers.
// Run with: node --test web/tests/browser/landing.mjs   (CHROME_BIN overrides the Chrome path)
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFile, execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdtempSync, readFileSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const webDir = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const chrome = process.env.CHROME_BIN ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const skip = existsSync(chrome) ? false : `no Chrome at ${chrome}`;

function buildSite() {
  const out = mkdtempSync(join(tmpdir(), "aq-landing-"));
  const source = join(webDir, "examples", "landing", "landing.aq");
  const style = join(webDir, "examples", "landing", "style.css");
  execFileSync("cargo", ["run", "-q", "--example", "build_site", "--", source, out, style], { cwd: webDir, stdio: "pipe" });
  return out;
}

function serve(root) {
  const server = createServer((req, res) => {
    const path = new URL(req.url, "http://localhost").pathname;
    // A route such as /how is the directory how/ with an index.html, as any static server maps it.
    let file = join(root, path);
    if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
    if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) {
      res.writeHead(404, { "content-type": "text/plain" });
      res.end("not found");
      return;
    }
    const types = { ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8" };
    const type = types[extname(file)] ?? "application/octet-stream";
    res.writeHead(200, { "content-type": type });
    res.end(readFileSync(file));
  });
  return new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve(server)));
}

// Asynchronous on purpose: the server that answers Chrome runs in this same process.
async function dump(url) {
  const { stdout } = await execFileAsync(
    chrome,
    ["--headless=new", "--disable-gpu", "--no-sandbox", "--dump-dom", url],
    { encoding: "utf8", timeout: 60000 },
  );
  return stdout;
}

const routes = [
  ["/", "One language for coding agents"],
  ["/how", "How it works"],
  ["/start", "Get started"],
];

test("landing: every route renders its heading in Chrome and every link answers", { skip }, async () => {
  const root = buildSite();
  const server = await serve(root);
  const base = `http://127.0.0.1:${server.address().port}`;
  try {
    for (const [route, heading] of routes) {
      const html = await dump(base + route);
      assert.ok(html.includes(`<h1>${heading}</h1>`), `${route} shows its heading`);
      assert.ok(!html.includes("aqelith-runtime.js"), `${route} is static`);
    }
    for (const [route] of routes) {
      const html = await dump(base + route);
      for (const [, href] of html.matchAll(/<a href="([^"]+)"/g)) {
        const response = await fetch(base + href);
        assert.equal(response.status, 200, `link ${href} from ${route}`);
      }
      for (const [, href] of html.matchAll(/<link rel="stylesheet" href="([^"]+)"/g)) {
        const response = await fetch(base + href);
        assert.equal(response.status, 200, `stylesheet ${href} from ${route}`);
        assert.match(response.headers.get("content-type"), /^text\/css/, `stylesheet ${href} is css`);
      }
    }
  } finally {
    server.close();
  }
});
