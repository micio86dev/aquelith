# ADR-0009: v0 syntax subset (answers Q6, Q7)

- Status: **Accepted for v0** (decided by the project owner delegating the choice, 2026-11-05). Full grammar remains Proposed.
- Date: 2026-11-05

## Context

Q6 (operators, closures, string interpolation, attributes) and Q7 (error propagation) blocked M4. The grammar draft in [../language/GRAMMAR.md](../language/GRAMMAR.md) left them open.

## Decision for v0

- Operators: `+`, `-`, `*` (left-associative, `*` binds tighter), `==` (non-associative: `a == b == c` is an error, PAR005).
- No closures, no string interpolation, no attributes, no assignment, no `return`. The tail expression of a block is its value.
- No `?` operator. Recoverable errors will be `Result` values in a later version; v0 has no error values.
- Types in v0: `i32`, `u32`, `bool`, `unit`. Integer literals take their type from context; with no context they are `i32`.
- Equality operands: when the left operand is an integer literal and the right is not, the literal takes the right operand's type. This makes `0 == x` work for `x: u32`.

## Alternatives considered

- `?` now: rejected for v0; it needs error values, which v0 does not have.
- Full operator table with bitwise and comparison chains: rejected; adds ambiguity without need.

## Evidence

- Astra's audit: `n-1` without spaces failed to parse, so v0 accepts any spacing and tests it.
- The agent-repair concern in the directive (§4) is addressed by making a small grammar with one canonical formatting.

## Consequences

Everything listed as "not in v0" is a compile-time error with a stable code, not a silent fallback.
