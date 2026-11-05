# AQELITH Conformance Strategy

Status: **Proposed (M2).** Defines how the conformance levels in [../platforms/CONFORMANCE.md](../platforms/CONFORMANCE.md) are tested.

## 1. Corpus

- A shared corpus of AQELITH programs, each with: source, expected stdout, expected exit code, and optional expected diagnostics.
- Corpus grows with each feature; a feature's test is added in the same change as the feature.
- Programs are small, self-contained and deterministic (no wall-clock, no network unless the test declares a local fixture server).

## 2. Backends compared

Interpreter (reference) and every production backend run the same corpus. The interpreter is the reference; a backend that differs is wrong unless the spec says the behavior is target-defined.

## 3. Differential testing

Differences between backends are minimized automatically to a smallest failing program. The AICore audit recommended a differential gate from day one; AQELITH adopts it once two backends exist.

## 4. Reporting

Same report format as [../platforms/CONFORMANCE.md](../platforms/CONFORMANCE.md) §5: pass, fail, skip with reasons, toolchain version, commit.

## 5. Current status

Corpus: empty. Backends: none. Nothing in this strategy has run.
