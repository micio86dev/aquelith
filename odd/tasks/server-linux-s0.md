# Feature: server target on Linux, milestone S0 (IR)

Objective: implement S0 of [docs/platforms/SERVER_LINUX_PLAN.md](../../docs/platforms/SERVER_LINUX_PLAN.md) for the v0 function subset. Later milestones (S1 interpreter, S2 server language slice, S3 HTTP, S4 conformance, S5 security) are separate feature documents.
Branch: feat/web-dev-loop (current). Delivery strategy: ask-on-risk (default). Test runner: `cargo test` and `cargo clippy --all-targets -- -D warnings` in `compiler/`; `gga run` before each commit.
No new crates.

## Specs

R1. The checker records the type of every expression node, keyed by byte span, and returns it only for an accepted module.
R2. Lowering to a canonical IR is defined only for accepted modules; a rejected module gives no IR.
R3. Lowering is deterministic: the same source gives the same IR and the same text.
R4. Every accepted corpus file lowers; every rejected corpus file does not.
R5. Each expression node has its own span, so type lookup by span is sound.
R6. The IR and its status are documented in `docs/architecture/IR.md`, stating what is not covered.

## Tasks

T1 (R1, R5) Checker records expression types per function (`compiler/src/check.rs`). Route: inline. Risk: none (no diagnostic changes; the full suite is unchanged). Status: committed in the IR code unit; that unit passed `gga run`.
T2 (R2, R3, R4) IR and lowering (`compiler/src/ir.rs`, registered in `lib.rs`). Route: inline. Risk: none (new module; no public contract consumed by others). Status: committed in the IR code unit; that unit passed `gga run`; 6 tests in `compiler/tests/ir.rs` pass.
T3 (R6) Documentation (`docs/architecture/IR.md`). Route: inline. Status: written; `gga run` pending for this unit.
T4 (gates) Full compiler suite, clippy with warnings as errors, `gga run`. Status: suite 97/97 and clippy clean on the code unit; `gga run` pending for the documentation unit.

## Log

L1 (owner, 2026-11-06, Italian, verbatim, with English gloss): «Si fai tu» (approving the server plan); «Non ti fermare mai»; «E poi seguilo fino a completare tutto al 100% ben testato». Gloss: approved; do not stop; finish the work and test it.
L2 (owner, 2026-11-06, Italian, verbatim, with English gloss): «No! Non devi restringere nulla! Lo devi fare e basta!!». Gloss: do not narrow anything; just do it.
L3: The owner's goal is every target (all operating systems and devices). This document covers only the first milestone of the Linux server target, the next step in order. The other targets are not dropped: each needs its own feature document, to be written when its turn comes. Nothing has been narrowed in the goal itself.
L4: Decision: R1 uses the checker's own types (no second set of typing rules), because two sets would drift apart.
L5 (superseded by L6): integer overflow and the memory model were listed as blockers for S1.
L6: The stop hook recorded that the goal was not met, which is correct: S2–S5 and the other targets remain. The stop before S1 was not needed. Overflow is a best-practice default, checked arithmetic, recorded in docs/architecture/INTERPRETER.md and reversible by the owner. The memory model is not needed in S1: v0 functions allocate nothing (D3 is deferred until the first allocation exists).
L7: S1 done: reference interpreter (compiler/src/interp.rs), 10 tests, commit 81ae1a8, GGA passed. Compiler suite 107/107, clippy clean.
L8: Next: S2, the language slice for server programs (strings in functions, a request and response value, the net and io effects). It is a language design change; the minimal option will be chosen and documented, and each choice is reversible.
