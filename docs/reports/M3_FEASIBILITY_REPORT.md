# M3 Technical Feasibility Report

Date: 2026-11-05. Scope: directive §20 M3 ("minimal isolated prototypes only where architecture choices require evidence; discard prototypes that do not justify their complexity").

## 1. Prototypes built

| ID | Question | Location | Result |
|---|---|---|---|
| P1 | Do persistent ids survive edits, and do validated atomic patches reject stale or invalid input without writing? | `prototypes/stable-ids-patch/` | 8 of 8 tests pass; 2 mutations detected |
| P2 | Can a capability value be forged, and does the pattern hold in Rust? | `prototypes/capability-values/` | 5 of 5 tests and 1 compile-fail doctest pass; the doctest fails for the right reason |

Both are small, dependency-free Rust crates. They were run on the host because the code is project-owned (no third-party code, no network).

## 2. Findings

### P1: stable identities and patches

- Position-derived ids change after an insertion; persistent ids do not. This supports ADR-0004 on a toy graph.
- Stale and replayed patches are rejected without changing the store. Supports ADR-0002 and the patch API.
- Validation runs per op with one checker, so a partially valid patch cannot slip through. Supports the single-verdict rule (the Zerolang discrepancy).
- Limitation found: the hash used is not collision-resistant. The design already requires a cryptographic hash for integrity; the prototype does not provide one.

### P2: capability values

- A string token is forgeable by any code that knows the string. Supports ADR-0003 against the Orison-style design.
- Attenuation narrows and cannot widen. Compile-fail check confirms field privacy.
- **Important limitation found:** a public root-authority constructor lets any code in the process mint authority. Module privacy in Rust does not give "minted once by the host". AQELITH needs a rule for the root authority, now recorded as Q23 in OPEN_QUESTIONS.

## 3. What was NOT evaluated, and why

| Candidate prototype (directive §20 M3 examples) | Decision | Reason |
|---|---|---|
| Parser alternatives (hand-written vs parser generator) | Not built | Requires a grammar decision first (Q6). Comparing parsers on an undecided grammar would measure nothing useful. Deferred to M4. |
| IR representation benchmarks | Not built | No benchmark workload exists. A representation comparison without a workload would be an unsupported performance claim (directive §7.5). |
| Incremental compilation | Not built | Needs a compiler to be incremental. Deferred. |
| Web rendering (semantic DOM vs canvas) | Not built | Needs a browser test harness and a view-tree design. ADR-0006 rests on the directive constraint and the Naze audit; a browser prototype belongs to M6 with the renderer conformance suite. |
| Serialization and resumability | Not built | Needs the state model and a deployment-skew experiment, both M6. The skew question is kept open (Q12). |
| Capability enforcement at runtime | Not built | Needs a runtime. P2 covers the pattern only. |

Each deferred item is in the open questions list with the milestone that answers it.

## 4. Decisions informed

- ADR-0002, ADR-0004: supported on toy models; still Proposed.
- ADR-0003: supported on the pattern; the root-authority finding is new and recorded in ADR-0003.
- ADR-0005, ADR-0006, ADR-0007: no prototype evidence. Their status remains as written.

## 5. Measurements

None. The prototypes are too small to measure, and no measurement was asked for or would be meaningful. Benchmark methodology is in [../performance/BENCHMARK_METHODOLOGY.md](../performance/BENCHMARK_METHODOLOGY.md).

## 6. Mutation-check summary

On a scratch copy of P1:
- Without per-op validation: 1 test fails.
- Without the base-hash check: 2 tests fail.
P2's compile-fail doctest was confirmed to fail on E0451 (private field), not on an unrelated error.

## 7. Limitations of this feasibility work

- Two toy models, each a few hundred lines.
- No formatter, no parser, no text round trip.
- Single machine, single day.

## 8. Recommendation

Proceed to M4 with: ADR-0002 (text authority) and ADR-0004 (persistent ids) as working assumptions, the formatter round trip as the first M4 test, and a decided rule for the root capability (Q23). Revisit ADR-0003 once that rule is decided.
