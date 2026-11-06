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
| MCP handshake and tool list | stdio probe above | Yes |
| `aqelith_check` call | stdio probe | Yes, `ok: true` on a valid module |
| `claude mcp add` / `codex mcp add` syntax | `--help` of the installed CLIs | Yes |
| Real Claude Code session using the tools | not run | **Pending, by the user** |
| Real Codex session using the tools | not run | **Pending, by the user** |

To check the last two rows: open a session in each client, ask it to run `aqelith_check` on a small module with a deliberate type error, and confirm it reports the code `TYP001` rather than guessing.
