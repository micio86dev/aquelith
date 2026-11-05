# AQELITH Platform Conformance

Status: **Proposed (M2).** No conformance suite exists. This document defines what a platform must pass.

## 1. Conformance levels

| Level | Meaning | Required for |
|---|---|---|
| C0 | Builds and runs a hello-world program | Any "experimental" label |
| C1 | Passes the language conformance corpus for the core (no platform effects) | "Supported" for any target |
| C2 | Passes the interface conformance tests for every interface the target claims | "Supported" with those interfaces |
| C3 | Passes the renderer conformance suite (for UI targets) | "Supported" for UI |

## 2. Language corpus

Same programs, same expected output, on every target. Outputs are compared byte-for-byte. Differences in timing are excluded and listed as such. The corpus is part of [../testing/CONFORMANCE_STRATEGY.md](../testing/CONFORMANCE_STRATEGY.md).

## 3. Interface tests

For each interface in [PLATFORM_ABSTRACTION.md](PLATFORM_ABSTRACTION.md): success path, typed failure path, unsupported-capability path, and denial recorded in the audit list.

## 4. Renderer tests (UI targets)

- Semantic structure: the rendered tree matches the view tree.
- Keyboard navigation reaches every interactive element in document order.
- Focus is restored after route changes.
- Accessible names present for every interactive element.
- Browser E2E for the web target, using Playwright, per directive §14.

## 5. Reporting

A conformance run produces a machine-readable report with: target, toolchain version, commit, pass and fail counts, and every skipped test with its reason. A skipped test is never counted as a pass.

## 6. Current status

Zero targets conform. The reference competitors' platform claims are classified in the audits (for example Naze's Android path is a WebView wrapper, PI).
