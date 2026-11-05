# AQELITH Concurrency Model

Status: **Proposed (M2).** No runtime exists. The design must be validated by a prototype before M6; this document lists what must be decided and what the prototype must show.

## 1. Model

- **Tasks** are the unit of concurrency. A task is spawned inside a **scope**; the scope does not end until all its tasks end (structured concurrency).
- **Cancellation** is cooperative: a cancelled task receives a cancel signal at await points and must finish its cleanup. A scope cancels its children on exit from an error.
- **Shared state** is not shared by default. Data crosses tasks by ownership transfer or by an explicit synchronized type.
- **Await points** are explicit in the signature (`async`) and in the call site (`await`). Hidden suspension is not allowed.

## 2. Thread safety

- A type is `Send`-like (may move between tasks) or not, determined by its contents; the rule is computed, not declared. The AICore audit found data-race checking NV, so this rule needs a test corpus before it is trusted.
- Mutable shared access requires a synchronized wrapper. Its implementation is a library concern.

## 3. Platform scheduling

- The language does not require threads. Web targets use a single-threaded event loop; server and desktop targets may use a work-stealing pool. The same program must give the same observable results on both, except for timing. The conformance suite ([../platforms/CONFORMANCE.md](../platforms/CONFORMANCE.md)) tests this.

## 4. Effects

Async operations carry the `concurrency` effect in their signatures, consistent with [../security/EFFECT_SYSTEM.md](../security/EFFECT_SYSTEM.md).

## 5. What the prototype must show before this is accepted

- Scope exit waits for children; a child error cancels siblings.
- A cancelled task runs its cleanup exactly once.
- A deterministic test harness can replay an interleaving.

Status: none of these is prototyped yet. They are proposed acceptance criteria for the M6 runtime work (directive §20, M6), not current milestones.

## 6. Evidence and gaps

- Orison: single-threaded cooperative async, runtime not real (PI).
- AICore: async as poll-frame state machine, IE; data-race checking NV.
- Edict: no concurrency model in the audited scope.
