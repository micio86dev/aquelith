# Prototype P2: capability values versus string-keyed tokens

Status: **Prototype, Rust pattern test.** This does NOT test AQELITH's runtime, FFI or any language feature. It tests whether a capability value can be forged in Rust under two designs, to inform ADR-0003.

## Designs compared

1. **String-keyed token** (modelled on the Orison audit's description: in-memory tokens looked up by effect name). The token is a public string; any code that knows the string can construct it.
2. **Capability value** (ADR-0003): a struct with private fields, created only through an authority; attenuable to a subset; never widened.

## Run

```text
cd prototypes/capability-values && cargo test
```

No dependencies. The `compile_fail` doctest checks that a struct literal of `NetCap` outside its module is rejected.

## Results (2026-11-05, rustc 1.89.0)

- 5 of 5 integration tests pass.
- 1 of 1 `compile_fail` doctest passes, and was checked to fail for the right reason: `error[E0451]: field 'hosts' of struct 'NetCap' is private`.

## What it shows

1. A string token is forgeable: code that was never granted anything constructs the token from its text, and the table accepts it.
2. A capability refuses out-of-scope hosts with a typed error.
3. Attenuation can narrow a capability and cannot widen it.
4. Passing the capability as a parameter makes the dependency explicit in the signature.

## Finding: what Rust module privacy does NOT give

`Root::root()` is public. Any code in the same process can call it and mint authority. Rust cannot restrict "the host mints the root exactly once". This prototype therefore shows that **capabilities are unforgeable only for code that receives them from elsewhere**, not for code that can reach the authority constructor.

For AQELITH this becomes a language or runtime rule: the root authority is available only to the entry point, enforced by the compiler (for example, the entry-point signature is the only place that can call it) or by the runtime. That rule is an open question (see [../../docs/architecture/OPEN_QUESTIONS.md](../../docs/architecture/OPEN_QUESTIONS.md)) and is not decided here.

## What it does NOT show

- Runtime enforcement at each host call (ADR-0003 consequence): not modelled.
- Serialization, revocation, FFI crossing: not modelled.
- Ergonomic cost in real application code: not measured.
- Any performance property.

## Decision it informs

ADR-0003. Status stays **Proposed** until the root-authority rule is decided and the enforcement is prototyped in the M6 application work.
