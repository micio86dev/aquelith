# AQELITH (provisional name)

AQELITH is an experimental programming language designed so that coding agents can write, check and change programs with checks that report stable diagnostic codes. Every program is checked before it runs, and every diagnostic has a stable code.

> **Name status:** AQELITH is an internal codename. It has not been cleared legally or commercially. Do not publish packages, releases or a public repository under it. See [docs/branding/BRAND_DECISION.md](docs/branding/BRAND_DECISION.md).

**Scope of this version (v0).** This README describes only what is implemented and tested in this repository. Anything else is marked *planned*. Status by milestone: [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md).

## What works today

| Area | Works | Where |
|---|---|---|
| Language v0 | `i32`, `u32`, `bool`, `str`, `unit`; functions; effects; records; `page` items with text | [docs/language/GRAMMAR.md](docs/language/GRAMMAR.md), [docs/language/RECORDS.md](docs/language/RECORDS.md) |
| Compiler CLI | `check`, `fmt`, `query`, `explain`, `patch`, `ids`, `repair` | `compiler/` |
| Agent server | MCP over stdio: `aqelith_check`, `aqelith_format`, `aqelith_query`, `aqelith_explain`, `aqelith_patch` | `compiler/src/bin/aqelith-mcp.rs` |
| Agent skill | Language reference for agents | [skills/aqelith/SKILL.md](skills/aqelith/SKILL.md) |
| Web target | Static sites from `page` items, with minified HTML, a Content-Security-Policy, and a live-reload dev server | `web/` |
| Loopback host | `aqelith-serve <file.aq>` serves `fn handle(req: Request) -> Response` on 127.0.0.1, with Host, method and size limits | `compiler/src/bin/aqelith-serve.rs`, [docs/architecture/SERVE.md](docs/architecture/SERVE.md) |

*Planned, not built:* client-side state and events in pages, native backends (desktop, mobile, embedded), other operating systems, a package manager, runtime capabilities, a production or non-loopback server, TLS, and records in pages. The desktop, mobile and embedded targets have no backend yet.

## Install

You need Rust (stable). Build from the repository root:

```sh
cd compiler && cargo build --release --bins    # aqelith and aqelith-mcp
cd ../web && cargo build --release --examples  # build_site and dev_site
```

Binaries are in `compiler/target/release/` and `web/target/release/examples/`. Use absolute paths when you configure a client.

## Connect Claude Code

Three steps. Run them once on the machine where Claude Code runs (paths below are examples; replace them with yours):

```sh
# 1. The MCP server, for all projects
claude mcp add --scope user aqelith -- "/ABSOLUTE/PATH/aquelith/compiler/target/release/aqelith-mcp"

# 2. The language skill, loaded by Claude Code
mkdir -p ~/.claude/skills/aqelith && cp "/ABSOLUTE/PATH/aquelith/skills/aqelith/SKILL.md" ~/.claude/skills/aqelith/SKILL.md

# 3. Check it
claude mcp list    # aqelith should show "Connected"
```

Then add this to the `CLAUDE.md` of any project that uses AQELITH:

```markdown
## AQELITH (.aq files)
AQELITH is an experimental language. Read /ABSOLUTE/PATH/aquelith/skills/aqelith/SKILL.md before editing any .aq file.
Validate every change with the aqelith_check tool, or `aqelith check <file>`.
Use aqelith_patch for edits to existing functions; never hand-edit around a rejected patch.
```

For Codex, use `codex mcp add aqelith -- "<same path>"` and add the same block to `AGENTS.md`.

Smoke test without any client: the server answers one JSON message per line on stdin.

```sh
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./compiler/target/release/aqelith-mcp
```

## Write a page site

A minimal site is one `.aq` file. Every statement ends with `;`. A page header is `page <name> "<route>" {`, and a link is `link "<label>" "<route>";`:

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
    p "The first stable release.";
    link "All releases" "/";
}
```

Rules that trip up first attempts:

- Each page needs exactly one `title`. Text is printable ASCII without `"` or `\`.
- A link must name a route that exists in the same module.
- Comments are rejected. Put documentation in `docs/`.

Check, build, and preview:

```sh
compiler/target/release/aqelith check notes.aq
web/target/release/examples/build_site notes.aq dist [style.css]
web/target/release/examples/dev_site notes.aq [style.css] --port 8080   # live reload while you edit
```

`build_site` writes minified HTML with a Content-Security-Policy meta element, and copies a minified stylesheet if you pass one. `dev_site` serves on 127.0.0.1 only, from memory. It reloads the open page after each save, and shows a build error in a banner while keeping the last good page.

## Language in one page

- **Types:** `i32`, `u32`, `bool`, `str`, `unit`, and records. Literals take their type from context.
- **Functions:** the last expression of a block is its value; there is no `return`.
- **Effects:** a function that calls an effectful function must declare the effect in `uses`. A function with no `uses` is pure. The core effects are `io fs net time rand env proc concurrency ui unsafe`.
- **Diagnostics:** every error has a stable code (for example `TYP001`, `PAGE005`). Read the meaning with `aqelith explain <CODE>` rather than parsing the message.
- **Patches:** `aqelith patch` applies a JSON patch with preconditions. A rejected patch changes nothing. Dry-run first with `--dry-run`.

The full list of commands, codes and patch operations is in [skills/aqelith/SKILL.md](skills/aqelith/SKILL.md).

## Limits you should know

- No client-side state in pages, no imports, no closures, no methods on records. Strings in functions are `str` values with equality and return only; there is no concatenation.
- The web target is static. Any text is HTML-escaped; routes must be lowercase paths.
- Performance has not been measured on a reference machine. Build times and sizes in `docs/performance/` and `docs/reports/` are single samples on a development machine.
- Security review of the compiler and the web target is in progress; see `docs/security/`.

## Project layout

| Path | Content |
|---|---|
| `compiler/` | Language, checker, formatter, CLI, MCP server |
| `web/` | Static site backend, `build_site`, `dev_site`, browser tests |
| `skills/aqelith/` | Agent skill (`SKILL.md`) |
| `docs/` | Architecture decisions, language grammar, guides, reports, research |
| `odd/tasks/` | Feature documents for the work in this repository |

## Tests

```sh
cd compiler && cargo test
cd ../web && cargo test && node --test tests/reload.test.mjs
```

The landing browser test needs Google Chrome: `node --test tests/browser/landing.mjs`.

## Contributing

The engineering contract is [AGENTS.md](AGENTS.md). Use conventional commits, run `gga run` before a change is complete, and do not claim speed or safety without a reproducible measurement.

## License

Apache License 2.0. See [LICENSE](LICENSE).
