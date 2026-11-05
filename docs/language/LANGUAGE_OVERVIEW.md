# AQELITH Language Overview

Status: **Proposed (M2).** Nothing here is implemented. Syntax shown is illustrative and will be fixed by the parser prototype and the M4 foundation, not by this document.

## 1. Purpose

A single programming model for frontend, backend, full-stack, real-time, desktop, mobile, edge and embedded targets. The language core does not depend on any operating system, renderer or browser API. Platform features enter only through declared capabilities and target interfaces (see [../platforms/PLATFORM_ABSTRACTION.md](../platforms/PLATFORM_ABSTRACTION.md)).

## 2. Design rules (from the master directive)

1. Human-readable source is authoritative for authoring; the canonical IR is derived from it deterministically ([ADR-0002](../adr/ADR-0002-source-authority.md)).
2. Small set of canonical forms: one way to express each concept.
3. Explicit mutability, explicit effects, explicit capabilities.
4. Errors are values; no exceptions in the core.
5. Unsafe escape hatches are explicit, auditable and restricted (see [MEMORY_MODEL.md](MEMORY_MODEL.md)).
6. Every feature names its tests and its docs before it is marked implemented.

## 3. Feature inventory and status

| Area | Proposed design | Status | Where specified |
|---|---|---|---|
| Modules, imports | Explicit module path, no implicit globals | Proposed | this document |
| Functions, types, structs | Nominal records; algebraic data types | Proposed | [TYPE_SYSTEM.md](TYPE_SYSTEM.md) |
| Interfaces / traits | Coherence rule to be decided | Open question | [TYPE_SYSTEM.md](TYPE_SYSTEM.md), [../architecture/OPEN_QUESTIONS.md](../architecture/OPEN_QUESTIONS.md) |
| Generics | Parametric; no specialization in v1 | Proposed | [TYPE_SYSTEM.md](TYPE_SYSTEM.md) |
| Pattern matching | Exhaustive `match` | Proposed | [TYPE_SYSTEM.md](TYPE_SYSTEM.md) |
| Errors | Result-style values, typed error sets | Proposed | [ERROR_MODEL.md](ERROR_MODEL.md) |
| Async and concurrency | Structured tasks, cancellation scopes | Proposed | [CONCURRENCY_MODEL.md](CONCURRENCY_MODEL.md) |
| Memory | Single owner, borrows, no GC in core | Proposed; borrow rules open | [MEMORY_MODEL.md](MEMORY_MODEL.md) |
| Effects and capabilities | Effects in signatures; capabilities as values | Proposed | [../security/EFFECT_SYSTEM.md](../security/EFFECT_SYSTEM.md), [../security/CAPABILITY_MODEL.md](../security/CAPABILITY_MODEL.md) |
| Components, state, events, routing, data, validation, contracts | Application model | Proposed | [../app/APPLICATION_MODEL.md](../app/APPLICATION_MODEL.md) |
| Foreign function interface | Explicit `extern` with effect declaration | Proposed | [../security/SECURITY_ARCHITECTURE.md](../security/SECURITY_ARCHITECTURE.md) |

## 4. Illustrative shape (not final)

```text
module billing.invoice

type Money = { cents: u64, currency: Currency }

fn total(lines: List[Line]) -> Result[Money, PricingError]
    uses pure
{
    ...
}
```

Choices shown here are placeholders to be tested for readability, LLM error rate and parse ambiguity. The directive requires that syntax be chosen empirically, so the decision is deferred to the grammar and parser prototype (see [GRAMMAR.md](GRAMMAR.md)).

## 5. Explicit non-goals for v1

- No macros that run arbitrary code at compile time.
- No implicit global mutable state.
- No dependency on a single renderer or on a browser DOM in the core.
- No universal resumability (see [../app/RESUMABILITY.md](../app/RESUMABILITY.md)).
- No promise of memory safety for code inside an `unsafe` block.

## 6. Evidence this design relies on

Summarized from [../research/competitors/](../research/competitors/): explicit effects in signatures (Orison, AICore: both IT, with limits), graph-backed agent edits (Zerolang: IT for patches, PI for identity), structured diagnostics with stable codes (Astra, AICore, Edict: IE/IT). Gaps are listed in [../strategy/FEATURE_ADOPTION_MATRIX.md](../strategy/FEATURE_ADOPTION_MATRIX.md).
