# AQELITH Security Guarantees

Status: **Proposed (M2).** This document separates what each layer can guarantee. A guarantee listed here is a target until the test named in its row exists and passes. Until then its status is **not yet guaranteed**.

## 1. Compile-time guarantees (target)

| Guarantee | Enforced by | Test that would fail | Status |
|---|---|---|---|
| Calls with an undeclared effect are rejected | effect checker | a call-path fixture with a missing effect | not yet guaranteed |
| Use after move is rejected | ownership checker | move-then-use fixture | not yet guaranteed |
| Capability values cannot be constructed outside their authority module | visibility rules | `compile_fail` fixture (prototype P2 demonstrates this pattern in Rust) | prototype-level evidence only |
| Exhaustive matches | match checker | non-exhaustive fixture | not yet guaranteed |

## 2. Runtime guarantees (target)

| Guarantee | Enforced by | Status |
|---|---|---|
| A host call without a valid capability token is denied | runtime host interface | not yet guaranteed |
| Denials are recorded | runtime audit list | not yet guaranteed |
| Contracts that are checked at run time fail loudly, never silently pass | contract runtime | not yet guaranteed |

## 3. Operating-system guarantees

Depend on the OS and are not provided by AQELITH itself. Where a sandbox is used it is documented per platform in [../platforms/TARGET_MATRIX.md](../platforms/TARGET_MATRIX.md), and the platform's documented limits apply.

## 4. Guarantees that cannot be made

- No memory safety for `unsafe` code or foreign code.
- No protection against a compromised operating system, browser engine, or hardware.
- No protection against logic errors in authorization decisions made by the application.
- No guarantee that a dependency with declared capabilities behaves as declared unless the runtime enforces that declaration for the operation in question.
- No claim that a generated application is secure as a whole; the threat model lists what is and is not addressed ([THREAT_MODEL.md](THREAT_MODEL.md)).

## 5. Evidence required before any "secure" statement is published

- The guarantee's test passes in CI.
- A security test corpus (fuzzing, injection, traversal) runs with results attached.
- An independent review (directive §20, GGA plus the security assessment) is recorded.

Until these exist, no document or release may use the word "secure" to describe AQELITH.
