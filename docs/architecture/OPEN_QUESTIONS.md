# AQELITH Open Technical Questions

Status: **Living document, created at M2.** Each question names the milestone that must answer it and the evidence required.

| ID | Question | Answered by | Evidence required |
|---|---|---|---|
| Q1 | Coherence rule for interfaces/traits | M4 | Test corpus of overlapping implementations |
| Q2 | Integer overflow: checked by default or wrapping with diagnostic | M3 follow-up | Cost measurement of checked arithmetic on a loop benchmark |
| Q3 | Lifetime annotations needed in v1, or lexical borrows enough | M4 | Corpus of realistic programs; count of rejected patterns |
| Q4 | Whether v1 ships `unsafe` | M4 | Use cases from the reference app |
| Q5 | Unicode identifiers | M4 | Decision only |
| Q6 | Operator precedence, closures, string interpolation, attributes | M4 | **Answered for v0 by ADR-0009** (implemented, tested) |
| Q7 | Error propagation syntax (`?` or explicit) | M4 | **Answered for v0 by ADR-0009**: no `?` in v0; a later version adds it with error values. Agent repair-rate experiment still open |
| Q8 | Random 128-bit ids vs per-agent id ranges | M5 | Collision test under concurrent agents |
| Q9 | Merge of independently created nodes | M5 | Merge prototype |
| Q10 | Formatter lossless round trip for all semantic data | M4 | **Answered for v0 by ADR-0010**: no comments in v0, so the formatter is a pure function of the tree; idempotence tested over the corpus |
| Q11 | Serialization of capabilities across resume | M6 | Not planned to serialize (ADR-0003); confirm |
| Q12 | Deployment skew strategy for resumable apps | M6 | Experiment in RESUMABILITY §8 |
| Q13 | Reactivity mechanism for web (signals, compiled updates, both) | M6 | Benchmark on reference web app |
| Q14 | Native production backend | M7 | Benchmark of at least two candidates (ADR-0005) |
| Q15 | Native windowing libraries for desktop | M7 | Platform spike |
| Q16 | Signing scheme for packages and provenance | M8 | Threat-model review |
| Q17 | Whether user-declared effects are in v1 | M4 | Corpus of library use cases |
| Q18 | Concurrency data-race checking strategy | M6 | Test corpus; AICore audit found NV |
| Q19 | Runtime capability enforcement across FFI | M6 | Sandbox test corpus |
| Q20 | Embedded profile scope (no-heap) | M7+ | Feasibility study |
| Q21 | Whether the server target shares the dev interpreter | M6 | Benchmark and conformance comparison |
| Q22 | Reference machine for budgets | M5 | Declared hardware purchased or chosen by the user (may need decision; cost not known) |
| Q23 | Who may mint the root capability (only the entry point, enforced how?) | M4 | **Decided in ADR-0011** as a rule; enforcement (`CAP001`) not implemented until capabilities exist (M6) |
