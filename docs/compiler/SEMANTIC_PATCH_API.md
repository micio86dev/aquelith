# AQELITH Semantic Patch API

Status: **Proposed (M2).** Prototype P1 ([../../prototypes/stable-ids-patch/README.md](../../prototypes/stable-ids-patch/README.md)) tests the precondition and rejection rules in isolation.

## 1. Principle

An agent proposes a change; the compiler decides whether it applies. Syntactically valid JSON is not enough: every patch is checked against the current graph state, and a stale patch is rejected without touching storage (directive §10.3).

## 2. Patch shape

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

`rename`, `replace`, `insert`, `delete`, `set-field`, `add-effect`, `remove-effect`. Anything else requires a new API version. Structural rewrite-by-example is deferred (adoption matrix row 10).

## 6. Evidence

- Zerolang: graph-hash and node-hash preconditions, rejection leaves storage byte-identical (IT in the audit, reproduced). Its identity was position-derived; AQELITH uses persistent ids ([STABLE_IDENTITIES.md](STABLE_IDENTITIES.md)).
- Edict: node-id patches with atomic reject (IT).
- Orison: patch IR with dry-run and partial apply (IT). AQELITH keeps atomic all-or-none, which is safer for agents.

## 7. Limits

- A patch cannot prove that an intended behavior change is correct; it proves only that the result type-checks and preconditions hold. Behavior is tested, not patched.
- Rebasing a stale patch is out of scope for v1; the agent must re-query and resubmit.
