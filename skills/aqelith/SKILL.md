---
name: aqelith
description: Work with AQELITH v0 source files using the aqelith CLI: check, format, query, and explain diagnostics. Use when editing .aq files or when an agent needs structured compiler output. Covers only what v0 implements.
---

# AQELITH v0 (agent reference)

Status: **v0 subset.** Only what is listed here works. Anything else is a compile error, not a silent fallback. Source of truth: `compiler/`. This file is checked by `compiler/tests/skill.rs` against the real diagnostic codes.

## Commands

| Command | Use | Output |
|---|---|---|
| `aqelith check <file> [--json]` | Parse and type/effect check | Text lines `file:line:col: CODE: message`, or JSON `{"ok":bool,"diagnostics":[...]}` with `--json` |
| `aqelith fmt <file>` | Print canonical text | Stdout. Syntax only; ill-typed programs format too |
| `aqelith fmt <file> --write` | Rewrite the file in canonical form | Exit 0 on success |
| `aqelith query <file> <function>` | Callers and callees of one function | One JSON object: name, params, return, uses, calls, called_by |
| `aqelith explain <CODE>` | Meaning of a stable diagnostic code | One line |

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

## Workflow for an agent

1. Run `aqelith check file.aq --json`. Fix diagnostics by code, starting with the first one.
2. Run `aqelith fmt file.aq --write` after editing. The canonical form is what gets committed.
3. Before changing a function, run `aqelith query file.aq name` to see its callers and effects.
4. Never edit the output of `fmt` by hand to add features v0 does not have; the checker will reject them.

## Not available in v0 (do not promise)

Patch API, JSON-RPC daemon, MCP server, web runtime, native backends, package manager, capabilities at run time. These are planned in `docs/`.
