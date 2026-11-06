---
name: aqelith
description: Work with AQELITH v0 source files using the aqelith CLI: check, format, query, and explain diagnostics. Use when editing .aq files or when an agent needs structured compiler output. Covers only what v0 implements.
---

# AQELITH v0 (agent reference)

Status: **v0 subset.** Only what is listed here works. Anything else is a compile error, not a silent fallback. Source of truth: `compiler/`. The patch slice implements only the four ops listed below. This file is checked by `compiler/tests/skill.rs` against the real diagnostic codes.

## Commands

| Command | Use | Output |
|---|---|---|
| `aqelith check <file> [--json]` | Parse and type/effect check | Text lines `file:line:col: CODE: message`, or JSON `{"ok":bool,"diagnostics":[...]}` with `--json` |
| `aqelith fmt <file>` | Print canonical text | Stdout. Syntax only; ill-typed programs format too |
| `aqelith fmt <file> --write` | Rewrite the file in canonical form | Exit 0 on success |
| `aqelith query <file> <function>` | Callers and callees of one function | One JSON object: name, params, return, uses, calls, called_by |
| `aqelith patch <file> <patch.json> [--dry-run]` | Apply a semantic patch with preconditions (see below) | One JSON object: `accepted`, `dry-run` or `rejected` |
| `aqelith explain <CODE>` | Meaning of a stable diagnostic code | One line |
| `aqelith-mcp` | MCP server over stdio (JSON-RPC, one message per line) | Tools `aqelith_check`, `aqelith_format`, `aqelith_query`, `aqelith_explain`, `aqelith_patch`; text in, JSON out. Never reads or writes files |

Exit codes: `0` success; `1` diagnostics or function not found; `2` usage or I/O error.

## Language rules that matter

- The file starts with `module <name>`. Only `fn` items follow.
- Types: `i32`, `u32`, `bool`, `unit`. Integer literals take their type from context; with none they are `i32`.
- Effects: a fixed core set (`io fs net time rand env proc concurrency ui unsafe`). A function that calls an effectful function must declare that effect in `uses`. A function with no `uses` is pure.
- The last expression of a block (without `;`) is its value. There is no `return`.
- `==` does not chain: write `(a == b) == c`.
- No comments (`//` is LEX003). Put documentation in `docs/`.
- No closures, strings, assignment, `?`, imports, generics or traits in v0.

## Diagnostic codes

Stable codes. Read the meaning with `aqelith explain`; do not parse message text.

- LEX001, LEX002, LEX003: lexical errors (non-ASCII, unexpected character, comment).
- PAR001, PAR002, PAR005: syntax (unexpected token, nesting too deep, chained comparison).
- EFF001, EFF002: effects (undeclared effect in a call, unknown effect name).
- TYP001, TYP002, TYP003, TYP004: types (mismatch, literal range, arity, unknown type).
- RES001: unknown name or function.
- DUP001: duplicate function name.
- PAT001, PAT002, PAT003, PAT004, PAT005, PAT006: patch rejections (malformed document, stale base, failed precondition, unknown target, candidate fails the checker, source does not parse).

## Semantic patches (aqelith.patch.v1)

A patch is a JSON document. The compiler decides whether it applies; a rejected patch never changes the file.

```json
{"api":"aqelith.patch.v1","base":"graph:<hash>","ops":[
  {"op":"rename","target":"add1","expect":{"name":"add1"},"to":"inc"},
  {"op":"set-return","target":"main","expect":{"ret":"i32"},"to":"bool"},
  {"op":"add-effect","target":"main","expect":{"uses":["io"]},"to":"net"},
  {"op":"remove-effect","target":"main","expect":{"uses":["io","net"]},"to":"net"}
]}
```

- `base` is the graph hash from the current file. Read it from a rejection's `expected` field, or compute it by re-querying. It is computed over the canonical form, so spacing does not change it.
- `target` is a function name. Each `expect` must hold for the current program at the moment that op runs; ops run in order.
- `rename` also updates every call to that function. `add-effect` and `remove-effect` take effect names from the fixed core set.
- Ops are all-or-none. The candidate must pass the full checker and re-format to itself, or nothing is written.
- Use `--dry-run` first. Exit codes: `0` accepted or dry-run, `1` rejected, `2` usage or I/O error.
- Not implemented: `replace`, `insert`, `delete`, `set-field`, persistent ids. A target that is not a function name is rejected as PAT004.

## MCP server

`aqelith-mcp` speaks MCP protocol 2024-11-05 over stdin and stdout, one JSON-RPC message per line. Every tool takes source text as an argument. `aqelith_patch` returns a candidate with `status` "candidate" and never writes; a rejected patch is returned with `isError` true and its PAT code. To write, the agent passes the candidate to `aqelith patch` or `aqelith fmt --write`.

## Workflow for an agent

1. Run `aqelith check file.aq --json`. Fix diagnostics by code, starting with the first one.
2. Run `aqelith fmt file.aq --write` after editing. The canonical form is what gets committed.
3. Before changing a function, run `aqelith query file.aq name` to see its callers and effects.
4. Never edit the output of `fmt` by hand to add features v0 does not have; the checker will reject them.

## Not available in v0 (do not promise)

JSON-RPC daemon, web runtime, native backends, package manager, capabilities at run time. These are planned in `docs/`.
