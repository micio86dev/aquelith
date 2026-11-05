# ADR-0004: Stable identities: persistent opaque ids plus content hashes

- Status: **Proposed, provisional** until prototype P1 results are reviewed
- Date: 2026-11-05

## Context

The directive requires identity to survive formatting, renaming, moves and incremental compilation. Zerolang derives node ids from tree position and symbol ids from names; its own issue #348 on stable node identity is open, and the audit classifies stable identity PI. Edict uses author-assigned string ids and the audit notes that as a risk.

## Decision

Two concepts, never conflated:

1. **Persistent id**: opaque, assigned once at creation, never derived from position or name, never reused.
2. **Content hash**: a collision-resistant hash of a node's canonical content, used for staleness checks.

## Evidence from M3

Prototype P1 ([../../prototypes/stable-ids-patch/README.md](../../prototypes/stable-ids-patch/README.md)) demonstrates on a toy graph:

- inserting a node before another changes the position-derived id of the later node but not its persistent id;
- renaming keeps the persistent id and changes the content hash;
- a patch whose base hash is stale is rejected and the store is byte-identical afterwards.

This validates the rule on a toy model, not at project scale.

## Alternatives considered

- Position-derived ids (Zerolang): rejected; shown by P1 to break on insertion.
- Author-assigned names as ids (Edict): rejected; renames break references.

## Consequences

- Id generation must be collision-free across agents. Options (random 128-bit or per-agent ranges) remain open.
- Merges of independently created nodes need tooling (STABLE_IDENTITIES §6).
