# AQELITH Type System

Status: **Proposed (M2).** Describes intended rules; no checker exists yet.

## 1. Goals

Static, sound-by-design where feasible; inference local to function bodies with predictable boundaries; signatures are always explicit at module boundaries. The directive asks for "the smallest coherent model that can deliver strong guarantees", not a copy of Rust.

## 2. Core types

- Primitives: `bool`, fixed-width integers (`u8`..`u64`, `i8`..`i64`), `f32`, `f64`, `char`, `str` (UTF-8, immutable view), `unit`.
- Records: nominal, fields explicit. Tuples structural.
- Algebraic data types: sum types with exhaustive `match`.
- Collections: `List[T]`, `Map[K, V]` as library types, not built-in syntax.
- Option and Result: library types. No null.

Integer overflow semantics are an **open question** (the AICore audit lists it as planned): options are checked-by-default with explicit wrapping operators, or wrapping with a diagnostic. Decision deferred to M3 measurements of cost.

## 3. Inference

- Local variables: inferred from the initializer.
- Function parameters and return types: explicit.
- Generic arguments: inferred at call sites when unambiguous; otherwise a diagnostic with a repair suggestion.

## 4. Generics and interfaces

- Parametric polymorphism for functions and types.
- Interfaces describe required operations. Coherence (which implementation applies) is an **open question**. The AICore audit found generics and traits PI, so the design must not rely on a feature whose semantics are not yet pinned down by tests.

## 5. Effects in types

Function types carry an effect set. A function declares its effects with `uses`. Calls propagate effects transitively; a call with an effect not declared by its caller is a compile error with a call-path diagnostic (pattern from Orison E0420 and AICore's call-path diagnostic). Details: [../security/EFFECT_SYSTEM.md](../security/EFFECT_SYSTEM.md).

## 6. Refinement and contracts

Contracts (`requires`, `ensures`) are assertions that the checker may use. Static proving is **partial by design** in v1: the interval-level prover found in AICore is the model for what v1 may prove; anything unproven is checked at run time in debug builds and reported explicitly as "not proven", never silently assumed. The Edict corpus result (71.7% of contracts proven, with undecidable cases reported) shows why "undecidable" must be a first-class answer.

## 7. Soundness boundaries

- Within safe code, the type system must not allow a value of the wrong type to be used.
- Across FFI, the type system cannot guarantee the foreign side's behavior; FFI declarations are trusted boundaries and listed in the threat model.
- Unsafe blocks, if they exist, are explicit and do not weaken guarantees outside them.

## 8. Diagnostics

Every type error has a stable code, the expected and actual types, the source span, and optionally a machine-applicable fix. The error model is in [ERROR_MODEL.md](ERROR_MODEL.md).

## 9. Evidence

- Zerolang audit: a `u32` initialized from a string literal was accepted by the patch path and rejected only on the text path. The type rule above requires that both paths use the same checker, which is a design requirement, not a claim that Zerolang's implementation is wrong in general.
- Astra audit: several diagnostics had null line/column; the diagnostic contract requires spans.
