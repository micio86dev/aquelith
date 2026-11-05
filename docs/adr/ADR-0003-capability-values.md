# ADR-0003: Capabilities as values, not effect-name tokens

- Status: **Proposed, provisional** until prototype P2 results are reviewed
- Date: 2026-11-05

## Context

Competitors use "capability" for an inferred-effect lint (Astra), in-memory tokens looked up by effect name (Orison), or a clause repeating the effect set (AICore). None passes unforgeable capability values.

## Decision

Capabilities are typed values created only by the authority that owns the resource. Functions that need a resource take the capability as a parameter. Capabilities are attenuable (narrowed, never widened) and are never serialized (see [../security/CAPABILITY_MODEL.md](../security/CAPABILITY_MODEL.md)). Effects remain the static view and are checked separately ([../security/EFFECT_SYSTEM.md](../security/EFFECT_SYSTEM.md)).

## Evidence from M3

Prototype P2 ([../../prototypes/capability-values/README.md](../../prototypes/capability-values/README.md)) shows, in Rust:

- a string-keyed token can be copied and used by any code that knows the string;
- a capability whose constructor is private to its module cannot be created outside that module, verified by `compile_fail` doctests.

This validates the pattern in Rust. It does not validate AQELITH's runtime or FFI behavior.

## Alternatives considered

- Effect names only (Orison): rejected because the authority is forgeable.
- Ambient authority with a global registry: rejected by the directive (§6.1: "must not be accessible merely because a function can reference their names").

## Consequences

- **Finding from P2:** a public root constructor lets any code in the process mint authority. The rule for who may mint the root is open (OPEN_QUESTIONS Q23). Until it is decided this ADR is provisional.
- Call sites that need a resource carry an extra parameter; the ergonomic cost must be measured in the M6 application prototype.
- Capability values across FFI are an open question.
