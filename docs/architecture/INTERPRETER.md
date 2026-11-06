# Reference interpreter (milestone S1)

Status: **implemented for the v0 function subset.** Source: `compiler/src/interp.rs`. Tests: `compiler/tests/interp.rs` (10 tests). It runs the IR of [IR.md](IR.md); it is the reference semantics the plan asks for ([SERVER_LINUX_PLAN.md](../platforms/SERVER_LINUX_PLAN.md), S1).

## Decisions taken for S1

These are defaults chosen by the assistant on best-practice grounds. They are reversible: the owner can change any of them, and the change is then a recorded decision.

| Decision | Chosen | Why |
|---|---|---|
| Integer overflow | **Checked.** An operation whose result leaves the range of its type stops the run with `Overflow`. `u32` subtraction below zero is an overflow. | A silent wrap is a bug that nobody sees. Security is the second pillar in AGENTS.md, and checked arithmetic makes the error visible instead of hiding it. The open question in [TYPE_SYSTEM.md](../language/TYPE_SYSTEM.md) remains open for wrapping operators. |
| Declared effects | **Refused.** A function with `uses` does not run here: there is no host, so the call stops with `EffectNotAvailable`. | An effect must not run without a capability that grants it ([PLATFORM_ABSTRACTION.md](../platforms/PLATFORM_ABSTRACTION.md)). |
| Recursion | **Bounded** at 256 nested calls (`TooDeep`). | A runaway program must stop, not exhaust the process stack. |
| Memory model (plan decision D3) | **Not needed yet.** v0 functions allocate nothing: no strings, no records. | The model is decided when the first allocation exists. |

## Behaviour

- `run(program, name, args)` returns the value of the function, or a `RunError`. The tests cover arithmetic bounds, effects, recursion, bad calls and the zero-argument pure functions of the accepted corpus; the other paths are not yet tested for panics.
- Arguments are checked for count and type before the body runs.
- Every register is written before it is read. A read of an unset register is reported as `Malformed`, which an accepted program never produces.

## Limits

- No I/O, no network, no time: the effects are refused, not simulated.
- The interpreter is slow by design: it is the reference, not the fast path. No speed claim is made ([../performance/BASELINE.md](../performance/BASELINE.md)).
- Recursion depth is a fixed constant, not a measured stack budget.
