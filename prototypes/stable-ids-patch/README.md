# Prototype P1: stable identities and validated atomic patches

Status: **Prototype, toy model.** Not part of the AQELITH compiler. Tests ADR-0002 and ADR-0004 in isolation.

## What it does

- Stores nodes under persistent opaque ids; program order is kept separately (text position).
- Computes a content hash per node and a graph hash over program order and contents.
- Applies patches atomically: all ops or none. Each op has a precondition (name, value or content hash). Each patch has a base graph hash.
- Validates after every op with one checker (a stand-in for the full type check), so no path is weaker than the build.
- `dry_run` returns the predicted hash without writing.

## Run

```text
cd prototypes/stable-ids-patch && cargo test
```

No dependencies. Runs on the host because the code is project-owned.

## Results (2026-11-05, rustc 1.89.0, macOS arm64)

- 8 of 8 integration tests pass.
- Mutation check on a scratch copy:
  - Removing the per-op validation call: 1 test fails (`validation_rejects_out_of_range_u32_and_leaves_store_unchanged`).
  - Removing the staleness check: 2 tests fail (`stale_base_is_rejected_and_store_is_unchanged`, `replayed_patch_is_stale_after_it_applied_once`).

So the tests detect the defects they target.

## What it shows

1. A position-derived id (the Zerolang scheme, as described in its audit) changes for a node after an insertion before it. A persistent id does not.
2. Rename keeps the persistent id and changes the content hash.
3. A stale or replayed patch is rejected and the store is byte-identical (structurally equal) afterwards.
4. A failed precondition on a later op leaves earlier ops unapplied.
5. A value that fails validation is rejected at the op that caused it, with a stable code.
6. Ids are never reused after deletion.

## What it does NOT show

- Scale: the graph has 2 to 3 nodes. Cost of hashing and validation on large programs is unmeasured.
- Collision resistance: the hash is FNV-1a, which is not collision-resistant. Integrity use requires a cryptographic hash (ADR-0004; threat T3).
- The text round trip: the formatter loop of ADR-0002 is not exercised. This prototype tests the IR-side rules only.
- Real type checking: the validator is one range check.
- Concurrency: two agents patching at once are not simulated.

## Decision it informs

ADR-0004 (persistent ids plus content hashes) and the precondition rules in [SEMANTIC_PATCH_API.md](../../docs/compiler/SEMANTIC_PATCH_API.md). Status stays **Proposed** until the formatter round trip and a larger graph are tested.
