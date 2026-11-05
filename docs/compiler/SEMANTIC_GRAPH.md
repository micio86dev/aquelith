# AQELITH Semantic Graph

Status: **Proposed (M2).** The semantic graph is a query view over the canonical IR, not a separate store.

## 1. Node kinds

Modules, symbols, types, functions, components, state cells, effects, capabilities, dependencies, calls, data-flow edges, UI hierarchy nodes, contracts, platform requirements and tests. Each node has a persistent id and a content hash ([STABLE_IDENTITIES.md](STABLE_IDENTITIES.md)).

## 2. Edge kinds

`defines`, `refers-to`, `calls`, `depends-on`, `has-type`, `has-effect`, `requires-capability`, `flows-to` (data), `contains` (UI and syntax), `verified-by` (contract to test), `targets` (platform).

## 3. Queries (read-only)

- Symbols by name, by id, by kind.
- Callers and callees; transitive effect closure; blast radius of a change (what depends on a node).
- Type of a node; effects and capabilities of a function.
- Tests that cover a symbol (`verified-by`).
- Every query returns the graph hash it was computed against, so agents can detect staleness between query and patch.

## 4. Not in the graph

- Formatting and comments are in the text, not the graph, except as attributes that round-trip through the formatter.
- Runtime values. The graph is static.

## 5. Consistency rules

- Every edge endpoint exists in the same graph hash.
- Rebuilding the graph from the same source yields the same ids and hashes (determinism test).
- A graph built from an edited text that fails to parse is not published; the last valid graph is kept and the error is reported.

## 6. Evidence and cost

Zerolang exposes handle-addressed queries and reports stable identity as partly implemented (PI); see [../research/competitors/zerolang.md](../research/competitors/zerolang.md). The cost of graph construction on large projects is unmeasured; M3 does not measure it, and M5 must before any claim.
