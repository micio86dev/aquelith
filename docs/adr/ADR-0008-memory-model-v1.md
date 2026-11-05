# ADR-0008: Memory model for v1

- Status: **Proposed**
- Date: 2026-11-05

## Context

The directive asks for ownership, borrowing, explicit mutability, and a model smaller than Rust's. The AICore audit found ownership partial and lexical, no lifetimes, heap reclamation planned (PL) and `Box` absent. Astra's ARC scheme admits leaks. The directive forbids promising memory safety the implementation cannot prove.

## Decision

- Single owner, deterministic destruction at scope exit, no tracing GC in the core.
- Lexical borrows only: a borrow cannot outlive the owner's scope. Anything needing a lifetime annotation is a compile error with a repair suggestion. No lifetime syntax in v1.
- `unsafe`: deferred from v1 unless a concrete need appears; if added, it is explicit, audited, and counted.
- Memory safety is claimed only for safe code, and only after the tests in [../language/MEMORY_MODEL.md](../language/MEMORY_MODEL.md) §10 pass.

## Alternatives considered

- Rust-equivalent borrow checker with lifetimes: rejected for v1 on complexity grounds; the directive asks for the smallest coherent model.
- Garbage collection: rejected by the performance and platform constraints (embedded targets, predictable latency). Not measured; this is a design rationale, and M3 did not measure GC cost.

## Consequences

- Some patterns that Rust accepts with lifetimes will be rejected in v1. That is a stated limitation.
- The rule must be validated against realistic programs before M4 sign-off.
