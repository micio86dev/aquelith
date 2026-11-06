# Connecting a coding agent to AQELITH (local experiment)

Status: **local only.** Nothing here publishes the project. The name is provisional (see `docs/branding/BRAND_DECISION.md`).

Scope: this guide wires an agent to the **v0 compiler** (`aqelith` CLI and `aqelith-mcp` MCP server) and to the agent reference skill. It covers what the v0 subset can do today: check, format, query, explain, and semantic patches on `.aq` source text, including `page` items. Building a web site from pages is a separate step with `build_site` (see "Known limits").

Verified in this repository on 2026-11-06: the MCP stdio handshake (`initialize`, `tools/list`, `tools/call`) and the `claude mcp add` and `codex mcp add` syntax against the installed CLIs (Claude Code 2.1.318, Codex CLI 0.158.0). The end-to-end agent session in each client was **not** run by the author; see "Verification status".

## 1. Build the binaries

```sh
cd "<repo>/compiler"
cargo build --release --bins
```

This produces `target/release/aqelith` (CLI) and `target/release/aqelith-mcp` (MCP server, stdio). Use absolute paths in client configuration.

## 2. Smoke test before connecting anything

```sh
MCP="<repo>/compiler/target/release/aqelith-mcp"
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | "$MCP"
```

Expected: an `initialize` result with `serverInfo.name` `aqelith`, then five tools: `aqelith_check`, `aqelith_format`, `aqelith_query`, `aqelith_explain`, `aqelith_patch`.

## 3. Connect Claude Code

Register the server (user scope, so every project sees it):

```sh
claude mcp add --scope user aqelith -- "<repo>/compiler/target/release/aqelith-mcp"
```

Check it: `claude mcp list` should show `aqelith`. Inside a session, `/mcp` shows its tools.

Give Claude the language rules. Add to the project's `CLAUDE.md`:

```markdown
## AQELITH (.aq files)
AQELITH is an experimental language. Your training data does not contain it.
Before editing any .aq file, read <repo>/skills/aqelith/SKILL.md.
Validate every change with the aqelith_check MCP tool, or `aqelith check <file>`.
Use aqelith_patch for edits to existing functions; never hand-edit around a rejected patch.
```

Optional: copy `skills/aqelith/SKILL.md` into `~/.claude/skills/aqelith/SKILL.md` so it loads as a skill. The `description` field in its frontmatter is what triggers it.

## 4. Connect Codex

```sh
codex mcp add aqelith -- "<repo>/compiler/target/release/aqelith-mcp"
```

Check it with `codex mcp list`. Codex reads `AGENTS.md`, so add the same block as in section 3 to the project's `AGENTS.md` (or to `~/.codex/AGENTS.md` for all projects), pointing to the same `SKILL.md` path. This repository's own `AGENTS.md` is the engineering contract for the compiler, not an agent brief for writing AQELITH programs; keep the two apart.

## 4b. Write a first page file (read before writing any .aq)

The grammar of a page is strict. This file checks and builds (verified 2026-11-06):

```aq
module notes

page home "/" {
    title "Release notes";
    h1 "Release notes";
    p "Every release is listed here, newest first.";
    link "Version 1.0" "/v1";
}

page v1 "/v1" {
    title "Version 1.0";
    h1 "Version 1.0";
    h2 "What changed";
    p "The first stable release.";
    link "All releases" "/";
}
```

Rules that a first attempt usually gets wrong:

- A page header is `page <name> "<route>" {`. The name is a plain identifier (`home`), then the route as a string. `page "/" {` is rejected (PAR001).
- Every statement ends with `;`, including `title`, `h1`, `h2`, `p` and `link`.
- Link order is `link "<label>" "<route>";`. The label comes first. `link "/v1" "Version 1.0";` is rejected (PAGE003).
- Text is printable ASCII without `"` or `\`.

Check, then build. Run the commands from the directories shown (the build command uses the `web` crate):

```sh
"<repo>/compiler/target/release/aqelith" check notes.aq
cd "<repo>/web" && cargo run -q --example build_site -- /absolute/path/notes.aq /absolute/path/out
```

Notes for a first run:

- `cargo run` in `web/` writes its build files to `web/target/`. That folder is git-ignored (`target/` in `.gitignore`), so the repository stays clean.
- The check above uses the release binary from section 1. A debug build (`compiler/target/debug/aqelith`, made by `cargo build --bins` without `--release`) behaves the same; use whichever you built.

Live preview with reload on each save (work item `web-dev-loop` in `odd/tasks/`; checked in real Chrome on 2026-11-06, and covered by the server and client tests in `web/tests/dev_server.rs` and `web/tests/reload.test.mjs`):

```sh
cd "<repo>/web" && cargo run -q --example dev_site -- /absolute/path/notes.aq /absolute/path/style.css --port 8080
```

Open `http://127.0.0.1:8080/` in a browser and leave it open. After each save of the `.aq` or the `.css` file, the page reloads by itself. The delay is set by the constants in `web/src/dev.rs`: the file is polled every 250 ms, and a rebuild waits until the content has been unchanged for 100 ms (up to 20 such waits). So the reload comes between about 0.1 s and 2.3 s after the last write, plus the build time. This is the bound from the constants; it has not been measured on a reference machine. A source that does not check shows the diagnostic in a banner at the top, and the last good page stays under it; the next valid save reloads the page and removes the banner. The server listens on 127.0.0.1 only and answers only `GET` for its own host name.

## 5. What the agent gets

| Tool | Input | Use |
|---|---|---|
| `aqelith_check` | `source` | Diagnostics with stable codes (`TYP001`, `EFF001`, ...) |
| `aqelith_format` | `source` | Canonical text (the formatter is idempotent) |
| `aqelith_query` | `source`, `function` | Callers, callees, effects of one function |
| `aqelith_explain` | `code` | Meaning of a diagnostic code |
| `aqelith_patch` | `source`, `patch` | Verified candidate text; never writes a file |

The MCP server never reads or writes the disk. The agent writes files with its own tools, then runs `aqelith check` on the result.

The HTTP transport (`aqelith-mcp --http 127.0.0.1:PORT`, `POST /mcp`) is loopback-only and rejects non-local `Host` headers. Prefer stdio for these clients.

## 6. Known limits (read before testing with an agent)

- v0 types are `i32`, `u32`, `bool`, `unit`. Functions have no strings, no structs, no closures, no assignment, and no imports.
- String literals exist only in `page` items: printable ASCII, no `"` or `\`, no escapes (ADR-0015). A page holds `title`, `h1`, `h2`, `p` and `link`; the checker enforces PAGE001 to PAGE005.
- The web backend (`web/`) renders a module's pages from `.aq` source: `cargo run --example build_site -- <file.aq> <out> [style.css]`. Output is static HTML with no script. No images, lists or client-side behaviour in pages yet.
- Comments are rejected (LEX003). Documentation belongs in `docs/`.

Those limits decide what an experiment can measure today. Page items cover a static landing site (`web/examples/landing/`); anything beyond that needs a new language slice.

## 7. Verification status

| Step | Verified by | Result |
|---|---|---|
| Binaries build | `cargo build --release --bins` | Yes, 2026-11-06 |
| MCP handshake and tool list | stdio probe above, with stdin kept open | Yes, 2026-11-06 (the tools list was a multi-line reply until commit `38d2d05`, which stalled Claude Code) |
| `aqelith_check` call | stdio probe | Yes, `ok` on a valid module |
| `claude mcp add` / `codex mcp add` syntax | `--help` of the installed CLIs | Yes |
| Real Claude Code session using the tools | `claude -p` with the MCP server, deliberate type error | Yes, 2026-11-06: the tool was called and returned `TYP001` |
| Real Claude Code session writing a four-page site | `claude -p` in a fresh folder, see `docs/reports/MINI_SITE_TRIAL.md` | Yes, 2026-11-06: 6 tool calls, 18.5 s, 0 diagnostics, build passed |
| Real Codex session using the tools | not run | **Pending, by the user** |

Claude Code is connected through `claude mcp add --scope user aqelith`, and the skill is copied to `~/.claude/skills/aqelith/SKILL.md`. Both are in the user's configuration, not in the repository.
