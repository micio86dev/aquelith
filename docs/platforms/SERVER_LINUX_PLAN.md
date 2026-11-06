# Plan: server target on Linux (x86_64 first; ARM64 later)

Status: **Proposed. Not started.** Nothing in this plan is implemented. The target becomes "supported" only when every criterion in [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) section 3 holds, and the matrix in [TARGET_MATRIX.md](TARGET_MATRIX.md) says so.

## 1. Why this target, and what it depends on

Stage 2 in [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md): it is what full-stack applications need. It depends on three things that do not exist yet:

1. **A canonical IR.** The compiler checks source and formats it, but has no IR. [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) and [ADR-0005](../adr/ADR-0005-backend-strategy.md) name an IR and an interpreter as the first backend. Both are missing.
2. **Language features for server programs.** v0 has no strings in functions (string literals exist only in `page` items, ADR-0015), no structs, no closures, no imports, and the `io` and `net` effects are named but not implemented. A server program needs at least strings, a way to express a request and a response, and the `net` and `io` effects.
3. **A conformance suite.** [CONFORMANCE.md](CONFORMANCE.md) says none exists.

Without (1) and (2) there is nothing to serve. This plan therefore starts with them.

## 2. Decisions needed from the owner

These change the work. Each needs an answer before implementation starts.

- **D1. Interpreter or compiled artifact first?** An interpreter of the IR is the reference backend in ADR-0005. A compiled artifact needs a code generator, which is a much larger job. Its run-time speed is not measured, so no speed claim is made for either. Proposal: interpreter first; a compiled artifact is added later, and only then compared with a benchmark.
- **D2. Scope of the first server program.** Proposal: a loopback HTTP server that answers `GET` with static text, nothing else. It exercises `net`, strings and the response path, and is small enough to verify fully.
- **D3. Memory model on the server.** [MEMORY_MODEL.md](../language/MEMORY_MODEL.md) is a proposal. The interpreter needs a decision (garbage-collected or reference-counted) before any allocation is written.
- **D4. Which machines count.** Proposal: Linux x86_64 only, glibc, tested on the declared reference machine. ARM64 is out of this plan until an ARM64 runner exists to test it; it is then added as its own criterion.

## 3. Milestones with acceptance criteria

Each milestone ends with a committed work unit, GGA passing, and its criteria checked by a test that a reviewer can rerun.

| Milestone | What is built | Acceptance criteria |
|---|---|---|
| S0 | IR definition and deterministic lowering from the checked AST | Lowering the corpus twice gives identical IR. Every checked program lowers; no program that the checker rejects lowers. |
| S1 | Interpreter of the IR, reference semantics | The corpus runs with the expected results. Integer overflow behaviour matches the decision in [TYPE_SYSTEM.md](../language/TYPE_SYSTEM.md). |
| S2 | Language slice for server programs (strings, `net`/`io` effects, a request and response value) | The grammar and skill document the slice. The checker rejects every use of an effect that is not declared. |
| S3 | Loopback HTTP server, D2 scope | `curl http://127.0.0.1:PORT/` returns the expected body and status. Requests to other hosts are refused. A malformed request does not stop the server. The request head is limited to 8 KiB, as in the dev server ([web/src/dev.rs](../../web/src/dev.rs)). |
| S4 | Conformance tests for the target, in [CONFORMANCE.md](CONFORMANCE.md) form | All conformance tests pass on the declared reference machine (x86_64). The list of failures, if any, is in the matrix. |
| S5 | Security and runtime requirements documented | [SECURITY_ARCHITECTURE.md](../security/SECURITY_ARCHITECTURE.md) and the target's runtime requirements are written and reviewed. |

Performance is not a criterion until a baseline exists. The method of [performance/BASELINE.md](../performance/BASELINE.md) applies.

## 4. What this plan does not cover

- Desktop, mobile, embedded and the web backend beyond static pages.
- Package management and runtime capabilities.
- Public release. The brand gate in [BRAND_DECISION.md](../branding/BRAND_DECISION.md) applies.

## 5. Size

S0 to S5 is a multi-month effort for one person, and the estimate is not measured. It should be re-estimated after S0 and S1.
