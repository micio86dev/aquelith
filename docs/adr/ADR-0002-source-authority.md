# ADR-0002: Source authority

- Status: **Proposed**
- Date: 2026-11-05

## Context

The directive asks for human-readable source and a canonical typed IR, and says the two must not become silently inconsistent. Zerolang makes its graph the authority and keeps text as a projection, with explicit `export`, `import` and `verify-projection` commands; its audit reproduced that the graph path accepted a type-invalid literal that the text path rejected. Edict has no text syntax at all.

## Decision

- **The text source is the only authority for authored code.**
- The canonical IR is derived from text deterministically and is never edited by humans.
- Agent patches operate on the IR, are validated by the same checker as a build, and are written back as canonical text through the formatter. The written text is re-parsed and re-checked before the write is accepted ([../compiler/SEMANTIC_PATCH_API.md](../compiler/SEMANTIC_PATCH_API.md) §3).

## Alternatives considered

- Graph as authority with text as a projection (Zerolang): rejected for v1 because the reconciliation layer is a second authority in practice.
- Text only, no IR: rejected because agents need stable ids and queries (directive §10).

## Consequences

- The formatter must be lossless for everything the IR stores; comments and formatting that are not semantic must round-trip. Testing: formatter round-trip property test (see [../language/GRAMMAR.md](../language/GRAMMAR.md) §4).
- Patch latency includes parse and check. Must be measured (PERFORMANCE_MODEL).

## Evidence still missing

A prototype of the formatter loop does not exist. P1 tests precondition and rejection rules on a toy graph, not the formatter round trip.
