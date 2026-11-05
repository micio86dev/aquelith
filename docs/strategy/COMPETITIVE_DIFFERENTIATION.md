# Competitive Differentiation (M1)

Status: hypotheses derived from the audits in `docs/research/competitors/` and summarized in [COMPETITIVE_ANALYSIS.md](COMPETITIVE_ANALYSIS.md). None is a claim about AQELITH, which has no implementation yet. Each hypothesis names the evidence behind it, the risk, and the evidence M2/M3 must produce before it becomes a design commitment.

## D1. Human-readable source and a canonical graph, with one declared authority

- **Gap seen:** Zerolang makes the graph the truth and text a projection, and had to build an import/export/verify layer to reconcile them; its sync-state commands exist because the two diverge. Edict has no text syntax at all (ABS) and its audit names that as the biggest risk against its differentiator. AICore's IR is output-only, so "IR is canonical" does not hold for authoring (PI).
- **Hypothesis:** keep text as the authored form and derive the graph deterministically, or the reverse with a lossless text view, but state one authority and make the other verifiably derived.
- **Risk:** a lossless two-way mapping is hard; Zerolang's stale-projection handling shows the cost.
- **M3 evidence needed:** round-trip property tests on a prototype; a controlled agent-edit comparison, graph patches vs text edits, on the same tasks (nobody published one: Zerolang's token-efficiency claim is MC).

## D2. Mutations gated on the strongest validation level

- **Gap seen:** Zerolang's patch gate stops at shape validity: reproduced on the -Os build, a `u32` initialised with `"notanumber"` and an invalid call were applied with exit 0, `zero check` printed `ok`, while `check --json` said `buildable:false`.
- **Hypothesis:** a patch is accepted only if the result type-checks, effect-checks and (where applicable) satisfies contracts, and one verdict is authoritative.
- **Risk:** cost of full checking on every patch; needs incremental checking (Edict has it, IT).
- **M3 evidence needed:** patch-validation prototype with latency numbers on a realistic project.

## D3. Capabilities as values, not effect names

- **Gap seen:** Astra's "capabilities" are an inferred-effect lint (capability model PL). Orison's runtime tokens are in-memory strings looked up by effect name; the object-capability model is ABS. AICore duplicates `capabilities` and `effects` over the same fixed token set. No audited project passes capabilities as unforgeable values.
- **Hypothesis:** effects state what code may do; capability values state what it is allowed to touch, are passed explicitly, and are inspectable in the manifest and enforceable at runtime.
- **Risk:** ergonomics, and the compiler cannot make guarantees across FFI without a runtime boundary.
- **M3 evidence needed:** capability-enforcement prototype and a test corpus of escape attempts. Guarantees stay split into compile-time, runtime, OS-level, and not-guaranteed, as the directive requires.

## D4. Web UI that keeps the semantic DOM

- **Gap seen:** Naze, the only audited project with a real UI runtime, renders to Canvas2D with a hidden ARIA mirror that has no event listeners, is rebuilt on every render pass, and hides its static HTML with `display:none` once WASM starts. Orison's typed UI is signature-level (PI).
- **Hypothesis:** emit semantic HTML, keep it visible and focusable, and layer reactivity and resumability on top.
- **Risk:** Qwik-style resumability has an open question the reference audit could not close: how serialized state survives a new deployment (see `docs/research/REFERENCE_ECOSYSTEMS.md`). Fine-grained reactivity vs. DOM cost is unmeasured.
- **M3 evidence needed:** a small vertical slice with an accessibility test run and a deployment-skew experiment.

## D5. Evidence-linked claims

- **Gap seen:** every audited project had claims ahead of code or evidence: Naze (tier export, three outputs, pixel parity), Orison ("456 tests" vs 1097 passing, sub-100 µs on an 11-line fixture), Astra (closed source with a roadmap as proof, a benchmark harness bug admitted in its own page), Zerolang (token/memory/speed claims without committed benchmarks), Edict (a mock-repairer "benchmark").
- **Hypothesis:** a status matrix generated from CI, each row linking a passing test, plus benchmarks with committed scripts and raw results.
- **Risk:** discipline cost, not technical risk.
- **M3 evidence needed:** the generator and the first benchmark harness in place before any public performance sentence is written.

## What is not a differentiator

Stable error codes, JSON diagnostics, `explain` commands, node-id patches, dry-run, MCP servers and embedded skills already exist in several audited projects. AQELITH should adopt them (see the adoption matrix), not market them as novel.
