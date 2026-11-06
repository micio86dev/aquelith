# Records (S2): decision and syntax

Status: **implemented.** The owner asked for the best form for coding agents, and this is the form chosen. Tests: `compiler/tests/records.rs`. This document describes the v0 slice; anything not listed is not in the language.

## Why this form

Coding agents need a form that is explicit, typed and checkable without hidden behaviour. So: named fields, declared once at module level, built with a literal that names every field, read with a dot. No methods, no defaults, no inheritance, no generics, no implicit conversion.

## Syntax

```aq
module site

record Request {
    path: str,
}

record Response {
    body: str,
}

fn handle(req: Request) -> Response {
    Response { body: req.path }
}
```

- `record <Name> { <field>: <type>, ... }`. Fields are separated by commas; a trailing comma is allowed. A field type is `i32`, `u32`, `bool` or `str`, or another record type.
- A record literal is `<Name> { <field>: <expr>, ... }`. Every declared field appears exactly once; unknown or repeated fields are errors.
- A field is read with `<expr>.<field>`.
- Equality `==` is not defined on records. Records are passed, returned, bound with `let`, and read field by field.

## Diagnostics (new)

| Code | Meaning |
|---|---|
| `DUP001` | a record or a function is defined twice; a field is declared or set twice |
| `RES001` | unknown record, field or name |
| `TYP001` | a field value has the wrong type; a record is used where a primitive is expected; `==` on a record |
| `PAR001` | malformed record declaration, literal or field access |

## Not in this slice

Records are not yet covered by: methods, default values, update syntax, pattern matching, and records in pages. Nested records are supported: a field can have a record type.
