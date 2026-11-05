# AQELITH Capability Model

Status: **Proposed (M2).** The core choice (capability values, not effect-name tokens) is provisional until prototype P2 ([../../prototypes/capability-values/README.md](../../prototypes/capability-values/README.md)) and the M3 report confirm it.

## 1. Problem

Competitors use capabilities in three different senses:

| Project | What "capability" means there | Status in the audit |
|---|---|---|
| Astra | Inferred effect lint (`@pure`, `--deny`) | Capability model PL |
| Orison | Runtime token keyed by effect name, in-memory strings | IE; object-capability model ABS |
| AICore | `capabilities` clause repeating the effect token set | IE; duplicates `effects` |

None of them passes capabilities as values that cannot be forged. That is the gap AQELITH targets.

## 2. Design

- A **capability** is a value of a specific type (e.g. `NetCap`, `FsCap(root)`). It can only be created by the authority that owns the resource (typically the program entry point or the host).
- Functions that need a resource take the capability as an explicit parameter. A function that does not take one cannot use the resource.
- **Attenuation:** a capability can be narrowed (`FsCap(root)` to `FsCap(root/sub)`, read-only from read-write), never widened.
- Capabilities are not copyable unless declared so, and cannot be stored in global state.
- The manifest records which capabilities a package requires; the consumer grants them.

## 3. Relation to effects

Effects ([EFFECT_SYSTEM.md](EFFECT_SYSTEM.md)) describe what a function may do, for static checking and reporting. Capabilities describe what it is allowed to touch, and are enforced by the runtime. Both are needed: effects give the compile-time view, capabilities the authority. The AICore clause that repeats effects as capabilities is the anti-pattern this split avoids.

## 4. Evidence

- P2 demonstrates, in Rust, that a string-keyed token can be forged by any code that knows the string, and that a value with a private constructor cannot be forged outside its module (compile-fail doctests). This is evidence about the pattern, not about AQELITH's runtime.

## 5. Open questions

- Runtime enforcement for capabilities passed across FFI.
- Serialization of capabilities in resumable state: default is that capabilities are never serialized; they are re-granted on resume ([../app/RESUMABILITY.md](../app/RESUMABILITY.md)).
- Revocation: not in v1.
