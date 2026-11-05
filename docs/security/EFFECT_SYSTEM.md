# AQELITH Effect System

Status: **Proposed (M2).**

## 1. Effects

An effect is a named kind of observable interaction with the outside world or with mutable shared state. Initial set (proposed, to be validated against the platform matrix):

| Effect | Meaning |
|---|---|
| `pure` | No effects; the default when `uses` is absent |
| `fs` | Filesystem access |
| `net` | Network access |
| `time` | Reading the clock or sleeping |
| `rand` | Non-deterministic randomness |
| `env` | Reading environment variables |
| `proc` | Spawning processes |
| `concurrency` | Creating tasks |
| `ui` | Touching a renderer or the DOM |
| `unsafe` | Operations outside the checked language |

The AICore audit used a fixed set of eight effects and found no polymorphism; AQELITH keeps a fixed core set for v1 but adds a rule that library authors can define effect names through a registered, versioned declaration. Whether v1 includes user-declared effects is an open question.

## 2. Declarations and checking

- `uses` lists the effects a function may perform. Absence means `pure`.
- A call propagates the callee's effects to the caller; the caller's `uses` must cover them. A missing effect is a compile error with the call path that introduced it (pattern from the Orison E0420 and AICore call-path diagnostics).
- Effects are part of function types. Passing a function where an effect set is not allowed is an error.

## 3. Interaction with capabilities

Effects are checked statically. Capabilities are checked at the point of use. A function with `net` in `uses` still needs a `NetCap` value to actually connect. See [CAPABILITY_MODEL.md](CAPABILITY_MODEL.md).

## 4. Reports

The compiler emits an effect report per package: the set of effects each exported function may perform, and the dependency diff between versions (directive §6.2, and Orison AUD0001/2 as the model).

## 5. Limits

- The checker trusts foreign functions to declare their effects truthfully. A wrong declaration is the foreign-code trust boundary ([THREAT_MODEL.md](THREAT_MODEL.md), T4).
- Effects do not bound resource use (CPU, memory). That is covered by the performance budgets, not by this system.

## 6. Tests required

- Missing-effect fixture rejected with the call path.
- Effect report stable across two compilations of the same source (determinism).
- Dependency upgrade that adds `net` produces a diff entry.
