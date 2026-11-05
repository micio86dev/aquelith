# AQELITH Stable Identities

Status: **Proposed (M2).** Decision: [../adr/ADR-0004-stable-identities.md](../adr/ADR-0004-stable-identities.md). Prototype P1 tests the rename and position cases.

## 1. Requirements (directive §10.2)

Identity must survive formatting changes, refactoring, renaming, file moves, incremental compilation and branch changes.

## 2. Two separate concepts

| Concept | Purpose | How it is computed |
|---|---|---|
| **Persistent id** | Names a node across edits | Opaque, assigned once at creation, carried through rename and move, never derived from position or name |
| **Content hash** | Detects that a node's content changed | Collision-resistant hash of the node's canonical serialization |

Conflating these causes stale-edit bugs. Zerolang derives node ids from tree position and symbol ids from names; its own tracker issue (#348) on stable node identity is open, and the audit classified stable identity PI.

## 3. Allocation

- Persistent ids come from a per-project id space recorded in the project file. Two agents creating nodes concurrently must not collide: each agent gets its own id range, or ids are random 128-bit values. Choice is open; ADR-0004 proposes random 128-bit ids with a deterministic test mode.
- Ids are never reused after deletion.

## 4. Rename and move

Rename changes the name attribute; the persistent id is unchanged. Move changes the containing module; the id is unchanged. Prototype P1 demonstrates the first case with tests.

## 5. Formatting and comments

Formatting does not change ids or content hashes of semantic nodes; it changes only the text projection. Comments are attached to nodes and keep the node's id.

## 6. Branches

Ids are branch-local values that happen to be stable within a branch. Merging two branches where the same node was created independently produces two ids for logically similar code; that is a known limitation, and merge tooling must surface it. Not solved in v1.

## 7. Collision and integrity

Persistent ids are not security tokens. Content hashes used for staleness must not be the only integrity check where a hostile party can choose content (directive threat: T3 in [../security/THREAT_MODEL.md](../security/THREAT_MODEL.md)); use a collision-resistant function and document its choice.
