# Reference interpreter (milestone S1)

Status: **implemented for the v0 function subset.** Source: `compiler/src/interp.rs`. Tests: `compiler/tests/interp.rs` (11 tests). It runs the IR of [IR.md](IR.md); it is the reference semantics the plan asks for ([SERVER_LINUX_PLAN.md](../platforms/SERVER_LINUX_PLAN.md), S1).

## Decisions taken for S1

These are defaults chosen by the assistant on best-practice grounds. They are reversible: the owner can change any of them, and the change is then a recorded decision.

| Decision | Chosen | Why |
|---|---|---|
| Integer overflow | **Checked.** An operation whose result leaves the range of its type stops the run with `Overflow`. `u32` subtraction below zero is an overflow. | A silent wrap is a bug that nobody sees. Security is the second pillar in AGENTS.md, and checked arithmetic makes the error visible instead of hiding it. The open question in [TYPE_SYSTEM.md](../language/TYPE_SYSTEM.md) remains open for wrapping operators. |
| Declared effects | **Refused.** A function with `uses` does not run here: there is no host, so the call stops with `EffectNotAvailable`. | An effect must not run without a capability that grants it ([PLATFORM_ABSTRACTION.md](../platforms/PLATFORM_ABSTRACTION.md)). |
| Recursion | **Bounded** at 256 nested calls (`TooDeep`). | A runaway program must stop, not exhaust the process stack. |
| Work | **Bounded** at 1,000,000 instructions per run (`OutOfFuel`). | Depth alone does not bound time: a chain of functions that each call the next twice does 2^depth work while the depth stays low. Tested by `exponential_work_at_low_depth_stops_at_the_step_budget`. |
| Memory model (plan decision D3) | **Not needed yet.** String values are constants held in the IR; no function allocates at run time, and there are no records. | The model is decided when the first run-time allocation exists (for example concatenation). |

## Behaviour

- `run(program, name, args)` returns the value of the function, or a `RunError`. The tests cover arithmetic bounds, effects, recursion, bad calls and the zero-argument pure functions of the accepted corpus; the other paths are not yet tested for panics.
- Arguments are checked for count and type before the body runs.
- Every register is written before it is read. A read of an unset register is reported as `Malformed`, which an accepted program never produces.

## Limits

- No I/O, no network, no time: the effects are refused, not simulated.
- The interpreter is slow by design: it is the reference, not the fast path. No speed claim is made ([../performance/BASELINE.md](../performance/BASELINE.md)).
- Recursion depth and the step budget are fixed constants, not measured budgets.

## S2 slice: string values (done)

Implemented: the type `str`; string literals in functions; `str` parameters, return values, `let` values and calls; equality `==` on strings. Tests: `compiler/tests/strings.rs`.

Choices, reversible: strings are printable ASCII with no quote and no backslash, as in page items (ADR-0015); arithmetic on strings is a type error (`TYP001`); there is no concatenation, length or indexing.

Not done in S2, and why:
- **Request and response values.** These need record types, which v0 does not have. They are a language slice of their own, planned in [TYPE_SYSTEM.md](../language/TYPE_SYSTEM.md).
- **The `net` and `io` effects.** They are declared and refused at run time (section "Decisions"). Running them needs a host and a capability, which is the server milestone S3.
