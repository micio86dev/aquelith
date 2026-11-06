# AQELITH Semantic Patch API

Status: **Partly implemented (M5 slice v1).** `aqelith patch` implements §3 and §4 for eight ops. Targets are function names, or persistent function ids with `id:<hex>` when an index exists (§8). Prototype P1 ([../../prototypes/stable-ids-patch/README.md](../../prototypes/stable-ids-patch/README.md)) tested the precondition and rejection rules in isolation.

## 1. Principle

An agent proposes a change; the compiler decides whether it applies. Syntactically valid JSON is not enough: every patch is checked against the current graph state, and a stale patch is rejected without touching storage (directive §10.3).

## 2. Patch shape (the `sym:`/`node:` target forms are planned; function-name and `id:` targets are implemented, see §8)

```json
{
  "api": "aqelith.patch.v1",
  "base": "graph:<hash>",
  "ops": [
    { "op": "rename", "target": "sym:<opaque-id>", "expect": { "name": "old" }, "to": "new" },
    { "op": "replace", "target": "node:<opaque-id>", "expect": { "hash": "<content-hash>" }, "with": { "...": "..." } }
  ]
}
```

Each op carries a precondition. The whole patch is atomic: all ops or none.

## 3. Validation pipeline

1. Parse the patch; reject unknown `api` version or unknown op with a stable code.
2. Check `base` equals the current graph hash. If not, reject as stale. Do not attempt to rebase silently.
3. Check each op's precondition (name, content hash, field value). Reject with expected and actual values.
4. Apply ops to a copy of the IR.
5. Run the same checker as a full build (no weaker path). Reject if the result is not buildable, with the diagnostics.
6. Format to canonical text, re-parse, re-check. Only then write (atomic write, temp file plus rename).

Step 5 and 6 are the lesson from the Zerolang audit: its patch path accepted type-invalid literals while its check path reported `ok`. AQELITH requires one verdict from one checker.

## 4. Responses

- `accepted`: new `base` hash, list of touched node ids, validation status per level.
- `rejected`: stable code, the failing op index, expected and actual, and whether the rejection was staleness, precondition, or validation.
- `dry-run`: same as accepted without writing; required and tested.

## 5. Operations (v1 proposal)

Implemented ops are listed in §8; `set-field` in this list is planned, not built.

`rename`, `replace`, `insert`, `delete`, `set-field`, `add-effect`, `remove-effect`. Anything else requires a new API version. Structural rewrite-by-example is deferred (adoption matrix row 10).

## 6. Evidence

- Zerolang: graph-hash and node-hash preconditions, rejection leaves storage byte-identical (IT in the audit, reproduced). Its identity was position-derived; AQELITH uses persistent ids ([STABLE_IDENTITIES.md](STABLE_IDENTITIES.md)).
- Edict: node-id patches with atomic reject (IT).
- Orison: patch IR with dry-run and partial apply (IT). AQELITH keeps atomic all-or-none so a rejected patch never leaves partial state (a design rationale, not a measured comparison).

## 7. Limits

- A patch cannot prove that an intended behavior change is correct; it proves only that the result type-checks and preconditions hold. Behavior is tested, not patched.
- Rebasing a stale patch is out of scope for v1; the agent must re-query and resubmit.

## 8. Implemented slice (M5, v1)

Implemented in `compiler/src/patch.rs` and the `aqelith patch <file> <patch.json> [--dry-run]` command. Tests: `compiler/tests/patch.rs` (25), `compiler/tests/ids.rs` and repair in `compiler/tests/repair.rs` (5).

- **Targets** are function names, or `id:<32 hex digits>` resolved through the sidecar index `<file>.ids.json` (ids.rs, ADR-0004, function level only). A target that matches nothing is rejected as `PAT004`; an `id:` target without an index is `PAT001`. A rename keeps the id.
- **Base** is `graph:` plus FNV-1a 64 over the canonical text. It detects staleness, not tampering. Spacing in the source does not change it.
- **Ops**: `rename` (also rewrites callers), `set-return`, `set-param-type`, `add-effect`, `remove-effect`, `delete`, `replace` and `insert` at function level. `delete` and `replace` need `expect.hash`, the graph hash of the function's canonical text (`fn_hash`). `replace` and `insert` take one function definition as source text in `with`; it is parsed and then checked as part of the whole candidate. Each carries an `expect` precondition checked in order against the program after the earlier ops.
- **Verdict**: the candidate is formatted and checked by the same `check_source` as a normal build, and must re-format to itself. A rejected patch writes nothing.
- **Write**: temporary file in the same directory, then rename. A dry run writes nothing.
- **Codes**: PAT001 malformed, PAT002 stale base, PAT003 precondition, PAT004 unknown target, PAT005 candidate fails the checker, PAT006 source does not parse.
- **Repair**: `aqelith repair <file>` prints a patch of `add-effect` operations for effects that calls need, repeated to a fixed point and verified by `patch::apply` before it is printed (repair.rs).
- **Not implemented**: statement or expression level edits, node-level persistent ids, repair of type errors, graph hash in the other responses (only in `patch` responses).
- **Evidence**: mutation checks on scratch copies. Removing the full-checker step fails 4 patch tests; removing the base guard fails 3; ignoring the delete hash fails 1; a one-round repair fails 2 tests.
