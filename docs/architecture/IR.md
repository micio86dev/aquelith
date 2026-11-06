# Canonical IR (milestone S0)

Status: **implemented for the v0 function subset.** Source: `compiler/src/ir.rs`. Tests: `compiler/tests/ir.rs`. This document describes what exists; the wider IR planned in [ARCHITECTURE.md](ARCHITECTURE.md) and [ADR-0005](../adr/ADR-0005-backend-strategy.md) is not built.

## Scope

- Input: a module the checker accepts. `lower_module` returns `LowerError::Rejected` for any module with a diagnostic, and no IR is produced.
- Covered: functions over `i32`, `u32`, `bool` and `unit`; `let`; expression statements; the tail expression; the operators `+ - * ==`; calls; parentheses; the effect list of each function (carried, not interpreted).
- Not covered: page items (rendered by the web backend, not lowered), strings in functions (not in v0), and any execution of the IR (milestone S1).

## Types come from the checker

The checker records the type of every expression node, keyed by its byte span `(start, end)`, in `check::FnTypes`. Lowering reads those types and repeats no typing rule. A test checks that every expression node of the accepted corpus has its own span, so no two nodes can share a type by accident.

## Instructions

| Instruction | Meaning |
|---|---|
| `Const { dst, ty, value }` | `dst = value` |
| `Arith { dst, ty, op, lhs, rhs }` | integer `+`, `-` or `*` on operands of type `ty` |
| `Equal { dst, ty, lhs, rhs }` | `==` on operands of type `ty`; result `bool` |
| `Call { dst, ty, callee, args }` | call; `ty` is the callee's return type (`unit` allowed) |
| `Ret(Option<reg>)` | return a value, or nothing for `unit` |

Registers are numbered in the order the source is read: parameters first (`r0..`), then one new register per value-producing node. A `let` binds a name to the register of its value, with no copy. Operands are evaluated left to right.

## Determinism

The same source lowers to the same IR and to the same text. `Program::dump` prints a fixed form, and the test suite compares two lowerings of every accepted corpus file. Example:

```text
fn add1(r0: i32) -> i32 {
  r1: i32 = const 1
  r2: i32 = add r0, r1
  ret r2
}
```

## Acceptance for S0 (from [../platforms/SERVER_LINUX_PLAN.md](../platforms/SERVER_LINUX_PLAN.md))

| Criterion | Evidence |
|---|---|
| Every checked program lowers | the accepted corpus lowers (`every_accepted_corpus_file_lowers_...`) |
| No program the checker rejects lowers | the rejected corpus gives `Rejected` (`every_rejected_corpus_file_...`) |
| Lowering is deterministic | two lowerings compare equal; the dump is fixed (`a_known_module_...`) |
| Types come from the checker, with unique spans | `every_expression_node_has_its_own_span...` |

## Known limits

- A lowering error other than `Rejected` (`MissingType`, `Unbound`) is a compiler bug, not a user error. The corpus tests would fail if one were reached.
- The IR has no control flow: v0 has no loops or conditionals.
- Integer overflow is not defined here; its semantics are an open question in [../language/TYPE_SYSTEM.md](../language/TYPE_SYSTEM.md). S1 must not pick it silently.
