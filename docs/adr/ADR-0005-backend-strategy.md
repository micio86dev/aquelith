# ADR-0005: Backend strategy

- Status: **Proposed**
- Date: 2026-11-05

## Context

Directive §7.4: do not assume WASM is faster than JavaScript; do not add LLVM, Cranelift, JIT or several codegen pipelines in the first implementation unless measurements justify them. Competitors span interpreters (Astra, Orison), LLVM (Astra, AICore via text IR and clang), direct native emitters without LLVM (Zerolang, IE), and WASM (Naze, Edict).

## Decision

1. **Development:** an interpreter over the canonical IR. It is the reference for conformance ([../testing/CONFORMANCE_STRATEGY.md](../testing/CONFORMANCE_STRATEGY.md)).
2. **Web:** generated JavaScript for the UI and glue. WebAssembly only for compute-heavy modules, selected by measurement.
3. **Native production backend:** deferred. Choose one after a benchmark on the reference machine compares at least two options (candidate set: direct native emission, LLVM through text IR as AICore does, Cranelift). No commitment now.
4. No JIT in v1.

## Alternatives considered

- LLVM from the start: rejected for v1; adds a large dependency before the IR is validated.
- WASM for everything: rejected; the directive forbids assuming it is faster, and the WebAssembly security documentation notes its limits ([../security/SECURITY_ARCHITECTURE.md](../security/SECURITY_ARCHITECTURE.md) §6).

## Consequences

- The IR must be backend-neutral. Any backend-specific shape is a bug in the IR design ([../compiler/CANONICAL_IR.md](../compiler/CANONICAL_IR.md)).
- Two execution paths (interpreter and generated JS) require differential testing.

## Evidence still missing

No measurements. The M3 feasibility report lists which backend comparisons are required.
