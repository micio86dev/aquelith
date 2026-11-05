# AQELITH Grammar (draft)

Status: **v0 subset implemented** in `compiler/src/parser.rs` (tests: `compiler/tests/corpus.rs`, `compiler/tests/robust.rs`). The rest of this draft is still Proposed. The v0 choices are recorded in [../adr/ADR-0009-v0-syntax-subset.md](../adr/ADR-0009-v0-syntax-subset.md) and [../adr/ADR-0010-comments-and-formatting.md](../adr/ADR-0010-comments-and-formatting.md).

What v0 implements: `module` header, `fn` items with typed parameters, optional `-> type`, optional `uses` list from the fixed effect set, blocks with `let` (with optional `mut` and type), expression statements, a tail expression, integer and boolean literals, names, calls, parentheses, and `+ - * ==` with the precedence stated in ADR-0009. Not in v0: comments, closures, string literals, `return`, assignment, `?`, multi-module imports, generics, traits.

## 1. Requirements the grammar must meet

- Unambiguous with bounded lookahead (LL(1) for the core subset is the target; LL(2) allowed where it removes real ambiguity).
- Exportable to a constrained-decoding grammar (GBNF/EBNF) for agents, from the same source of truth, with a CI check (lesson from Naze: [../research/competitors/naze.md](../research/competitors/naze.md)).
- Stable, versioned surface: the grammar version is part of the CLI contract.
- Error recovery at statement and item boundaries, producing structured diagnostics with spans.

## 2. Lexical structure

- Source is UTF-8. Identifiers are ASCII in v1 (`[A-Za-z_][A-Za-z0-9_]*`); Unicode identifiers are an open question.
- Newlines terminate statements; blocks use braces. No semantic indentation (the Astra indentation-to-brace migration in its own audit shows the cost of mixing the two).
- Comments: `//` line and `/* */` block (non-nesting). Doc comments `///` attach to the next item.

## 3. Draft productions (EBNF-like)

```ebnf
module       = "module" path NEWLINE { item } ;
item         = fn_item | type_item | import_item ;
import_item  = "import" path [ "as" IDENT ] ;
type_item    = "type" IDENT [ generics ] "=" type_expr ;
fn_item      = "fn" IDENT [ generics ] "(" [ params ] ")" [ "->" type_expr ]
               [ "uses" effect_list ] block ;
effect_list  = IDENT { "," IDENT } ;
params       = param { "," param } ;
param        = IDENT ":" type_expr ;
generics     = "[" IDENT { "," IDENT } "]" ;
block        = "{" { stmt } [ expr ] "}" ;
stmt         = let_stmt | expr_stmt ;
let_stmt     = "let" [ "mut" ] IDENT [ ":" type_expr ] "=" expr ;
expr_stmt    = expr ;
expr         = literal | IDENT | call | match_expr | block ;
call         = IDENT "(" [ args ] ")" ;
match_expr   = "match" expr "{" { arm } "}" ;
arm          = pattern "=>" expr ;
type_expr    = IDENT [ generics ] | "(" type_expr { "," type_expr } ")" | ... ;
```

Open points: operator precedence table, closures, string interpolation, attributes, the exact `uses` clause placement, and whether `Result` has a `?` operator (the Astra and Zerolang audits both show error-propagation syntax is a common agent failure point; decide with the error-model test corpus).

## 4. Tests this grammar must pass before M4 sign-off

- Accept corpus: every example in this directory once examples exist.
- Reject corpus: each error code in [ERROR_MODEL.md](ERROR_MODEL.md) has at least one minimal input.
- Round-trip: `format(parse(x)) == format(parse(format(parse(x))))` (idempotence).
- Grammar export: the generated GBNF accepts the accept corpus and rejects the reject corpus.
