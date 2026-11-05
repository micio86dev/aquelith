# AQELITH Memory Model

Status: **Proposed (M2).** The directive forbids promising memory safety the implementation cannot prove. This document separates what the design intends from what must be shown.

## 1. Allocation and deallocation

- Values have a single owner. When the owner goes out of scope, the value is released deterministically (scope-based destruction). No tracing garbage collector in the core.
- Heap allocation is explicit through library types (`Box`, `List`, `Map`). Stack allocation is the default for records.
- **Resource cleanup** runs at scope exit in reverse declaration order; cleanup must not fail silently.

The AICore audit found heap reclamation planned (PL), and Astra's own docs admit leaks in its ARC scheme. Both are the failure modes this design must test against.

## 2. Ownership transfer

- Assignment and passing a non-`copy` value by value moves it; the source becomes invalid, and use after move is a compile error.
- `copy` types are small value types declared explicitly.

## 3. Borrowing

- Shared borrows (`&T`) and exclusive borrows (`&mut T`), with the rule that a value cannot be mutated while shared borrows are live.
- **Open question:** whether v1 needs explicit lifetime annotations or can use lexical scoping only. The AICore audit found its borrow model lexical with no lifetimes (PI) and a cited dangling-reference gap (PL). v1 proposal: lexical borrows that cannot escape the scope that owns the value; anything that would need a lifetime annotation is a compile error with a repair suggestion, not an implicit rule.

## 4. Mutability

Mutability is explicit (`let mut`). Interior mutability is a library type; its use is listed in the effect report.

## 5. Threads and tasks

Values cross task boundaries by move or by synchronized wrapper. See [CONCURRENCY_MODEL.md](CONCURRENCY_MODEL.md).

## 6. FFI boundaries

Memory passed to foreign code is either borrowed for the call's duration or transferred explicitly. Foreign code is outside the guarantees (see [../security/SECURITY_GUARANTEES.md](../security/SECURITY_GUARANTEES.md)).

## 7. Unsafe

`unsafe` blocks, if included in v1, must be: explicit in source, listed in an audit report, restricted to named operations, and counted in the security status matrix. Whether v1 ships `unsafe` at all is an open question.

## 8. Embedded and no-std

A target profile can disable heap allocation entirely; library types that allocate are then compile errors for that target. This is a target conformance requirement ([../platforms/TARGET_MATRIX.md](../platforms/TARGET_MATRIX.md)).

## 9. What is NOT claimed

- No claim of memory safety for `unsafe` code or foreign code.
- No claim about performance of this model versus any other. Measurements belong to [../performance/](../performance/) and are required first.

## 10. Tests required before implementation is marked done

- Use-after-move rejected; double-free impossible by construction (type-level test).
- Borrow escape rejected with a stable code.
- Cleanup order test with a fixture that records the sequence.
- A sanitizer run (AddressSanitizer or equivalent) over the runtime once a runtime exists ([../testing/SECURITY_TESTING.md](../testing/SECURITY_TESTING.md)).
