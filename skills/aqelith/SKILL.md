---
name: aqelith
description: Work with AQELITH v0 source files using the aqelith CLI or its MCP server: check, format, query, explain diagnostics, and apply semantic patches with preconditions. Use when editing .aq files or when an agent needs structured compiler output. Covers only what v0 implements.
---

# AQELITH v0 (agent reference)

Status: **v0 subset.** Only what is listed here works. Anything else is a compile error, not a silent fallback. Source of truth: `compiler/`. The patch slice implements the eight ops listed below. This file is checked by `compiler/tests/skill.rs` against the real diagnostic codes.

## Commands

| Command | Use | Output |
|---|---|---|
| `aqelith check <file> [--json]` | Parse and type/effect check | Text lines `file:line:col: CODE: message`, or JSON `{"ok":bool,"diagnostics":[...]}` with `--json` |
| `aqelith fmt <file>` | Print canonical text | Stdout. Syntax only; ill-typed programs format too |
| `aqelith fmt <file> --write` | Rewrite the file in canonical form | Exit 0 on success |
| `aqelith query <file> <function>` | Callers and callees of one function | One JSON object: name, params, return, uses, calls, called_by |
| `aqelith patch <file> <patch.json> [--dry-run]` | Apply a semantic patch with preconditions (see below) | One JSON object: `accepted`, `dry-run` or `rejected` |
| `aqelith explain <CODE>` | Meaning of a stable diagnostic code | One line |
| `aqelith repair <file>` | Print a patch that adds the effects calls need (verified before printing); `status: clean` when none is needed | One patch document, or the clean status |
| `aqelith ids <file> [--write]` | Show the persistent ids of the functions in `<file>.ids.json`; `--write` saves the index | JSON: `api` and `functions` (id and name) |
| `aqelith-mcp` | MCP server over stdio (JSON-RPC, one message per line) | Tools `aqelith_check`, `aqelith_format`, `aqelith_query`, `aqelith_explain`, `aqelith_patch`; text in, JSON out. Never reads or writes files |

Exit codes: `0` success; `1` diagnostics or function not found; `2` usage or I/O error.

## Language rules that matter

- The file starts with `module <name>`. Then `fn` and `page` items follow.
- Types: `i32`, `u32`, `bool`, `unit`. Integer literals take their type from context; with none they are `i32`.
- Effects: a fixed core set (`io fs net time rand env proc concurrency ui unsafe`). A function that calls an effectful function must declare that effect in `uses`. A function with no `uses` is pure.
- The last expression of a block (without `;`) is its value. There is no `return`.
- `==` does not chain: write `(a == b) == c`.
- Nesting and operator chains are bounded at 128 levels (`MAX_DEPTH`), so a long `a + b + ...` chain is refused with `PAR002` before it can exhaust the compiler's stack. Split long sums into `let` bindings.
- No comments (`//` is LEX003). Put documentation in `docs/`.
- String literals (`"..."`, printable ASCII, no `"` or `\`) appear in page items and, as type `str`, in functions. A `str` value can be a parameter, a return value, a `let` value, and the operand of `==`. Strings have no other operations: no concatenation, no length, no closures, assignment, `?`, imports, generics or traits.
- Page items (ADR-0015): `page <name> "<route>" { ... }` with exactly these statements, each ending in `;`: `title "..."` (once), `h1 "..."`, `h2 "..."`, `p "..."`, `link "label" "/route"`. A route is `/` or a lowercase path without trailing slash. Pages are content only; they carry no markup, and a backend renders them. The formatter prints functions first, then pages.
  Fragment: `page home "/" { title "Notes"; h1 "Notes"; p "Text."; link "Next" "/next"; }`. A link must name a route of a page in the same module, so this fragment alone fails with PAGE005 until a page `"/next"` exists. The header is `page <name> "<route>"`: the name comes first, then the route. The link is `link "<label>" "<route>";`, label first. `page "/" {` (no name) and `link "/next" "Next";` (reversed) are rejected.

## Diagnostic codes

Stable codes. Read the meaning with `aqelith explain`; do not parse message text.

- LEX001, LEX002, LEX003, LEX004, LEX005: lexical errors (non-ASCII, unexpected character, comment, unterminated string; backslash, control character or non-ASCII byte in a string).
- PAR001, PAR002, PAR005: syntax (unexpected token, nesting too deep, chained comparison).
- EFF001, EFF002: effects (undeclared effect in a call, unknown effect name).
- TYP001, TYP002, TYP003, TYP004: types (mismatch, literal range, arity, unknown type).
- RES001: unknown name or function.
- DUP001: duplicate function or page name.
- PAGE001, PAGE002, PAGE003, PAGE004, PAGE005: pages (title missing or repeated, empty text, invalid route or link target, duplicate route, link to a missing route).
- PAT001, PAT002, PAT003, PAT004, PAT005, PAT006: patch rejections (malformed document, stale base, failed precondition, unknown target, candidate fails the checker, source does not parse).

## Semantic patches (aqelith.patch.v1)

A patch is a JSON document. The compiler decides whether it applies; a rejected patch never changes the file.

```json
{"api":"aqelith.patch.v1","base":"graph:<hash>","ops":[
  {"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"},
  {"op":"set-return","target":"main","expect":{"ret":"i32"},"to":"bool"},
  {"op":"add-effect","target":"main","expect":{"uses":["io"]},"to":"net"},
  {"op":"remove-effect","target":"main","expect":{"uses":["io","net"]},"to":"net"},
  {"op":"delete","target":"unused","expect":{"hash":"graph:<hash of unused>"}},
  {"op":"insert","target":"main","with":"fn zero() -> i32 {\n    0\n}\n"}
]}
```

- `base` is the graph hash from the current file. Read it from a rejection's `expected` field, or compute it by re-querying. It is computed over the canonical form, so spacing does not change it.
- `target` is a function name. Each `expect` must hold for the current program at the moment that op runs; ops run in order.
- `rename` also updates every call to that function. `add-effect` and `remove-effect` take effect names from the fixed core set.
- Each op is checked by the full checker on the final candidate, so deleting or renaming a function that a caller still uses is rejected as PAT005.
- Ops are all-or-none. The candidate must pass the full checker and re-format to itself, or nothing is written.
- Use `--dry-run` first. Exit codes: `0` accepted or dry-run, `1` rejected, `2` usage or I/O error.
- `delete` removes a function when `expect.hash` matches it. `replace` swaps a function for the one in `with` when `expect.hash` matches. `insert` places the function in `with` right after `target`. `with` must hold exactly one function definition. The hash is the graph hash of that function's canonical text; read it from a PAT003 rejection or from a re-query.
- Target by persistent id with `"target":"id:<32 hex digits>"`. The id index is `<file>.ids.json`, kept by `aqelith patch`; a rename keeps the id, a deleted function's id is not reused in practice (128-bit random ids). Id targets are not available through the MCP server (no file access).
- `set-param-type` changes one parameter's type: `"expect":{"param":"x","type":"i32"},"to":"u32"`.
- `aqelith repair <file>` prints a patch that adds the effects calls need (for example `uses net` on a caller). Apply it with `aqelith patch`. The proposal is verified before it is printed; a clean file gives status `clean`.
- Not implemented: expression-level edits. A target that is not a function name is rejected as PAT004.

## MCP server

`aqelith-mcp` speaks MCP protocol 2024-11-05 over stdin and stdout, one JSON-RPC message per line. With `--http 127.0.0.1:PORT` it serves `POST /mcp` instead; only loopback addresses are accepted, and a request whose `Host` header is not localhost gets 403. Every tool except `aqelith_explain` takes source text as an argument; `aqelith_explain` takes a diagnostic `code` (for example `TYP001`). `aqelith_patch` returns a candidate with `status` "candidate" and never writes; a rejected patch is returned with `isError` true and its PAT code. To write, the agent passes the candidate to `aqelith patch` or `aqelith fmt --write`.

## Workflow for an agent

1. Run `aqelith check file.aq --json`. Fix diagnostics by code, starting with the first one.
2. Run `aqelith fmt file.aq --write` after editing. The canonical form is what gets committed.
3. Before changing a function, run `aqelith query file.aq name` to see its callers and effects.
4. Never edit the output of `fmt` by hand to add features v0 does not have; the checker will reject them.

## Not available in v0 (do not promise)

JSON-RPC daemon, native backends, package manager, capabilities at run time. The web backend renders pages from `web/` only; mobile and desktop backends are planned in `docs/`.
