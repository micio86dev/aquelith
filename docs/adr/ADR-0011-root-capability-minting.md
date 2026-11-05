# ADR-0011: Who may mint the root capability (answers Q23)

- Status: **Accepted as a rule; enforcement not yet implemented** (decided by the project owner delegating the choice, 2026-11-05)
- Date: 2026-11-05

## Context

Prototype P2 showed that a public root constructor lets any code in a Rust process mint authority. Module privacy cannot express "minted once, by the host".

## Decision

1. The root capability is produced only by the runtime, and handed only to the program's entry function `main`, as an argument. No other function can name the root.
2. The compiler rejects any reference to the root-producing operation outside the entry function (diagnostic code reserved: `CAP001`).
3. Capabilities derived from the root are passed explicitly; they are never global.

## Status of enforcement

- v0 has no capabilities and no runtime, so this rule is recorded but **not enforced anywhere yet**.
- The compile-time check is planned for the first milestone that introduces capabilities (M6). Until it has a passing test, documents must not say the rule is enforced.

## Alternatives considered

- Keep a public constructor and rely on review: rejected; P2 showed it is forgeable.
- A runtime check that the root is minted once: rejected as the only mechanism; it is a late, dynamic defence and does not stop the code from being written.

## Evidence

P2 ([../../prototypes/capability-values/README.md](../../prototypes/capability-values/README.md)) and its compile-fail doctest.
