# AQELITH Error Model

Status: **Proposed (M2).** Defines the contract; no diagnostics are emitted yet.

## 1. Two kinds of failure

| Kind | Mechanism | Examples |
|---|---|---|
| Recoverable errors | Values of type `Result[T, E]` | file not found, invalid input, network timeout |
| Defects | Contract violation, unreachable state, internal invariant | failed `requires` at run time, index out of bounds in safe code |

The core has no exceptions. Defects terminate the current task with a structured report; they are not caught and resumed silently.

## 2. Error types

- `E` is an explicit type. Error sets are closed unions in signatures; a function that can fail with a new kind of error changes its signature.
- Every error value carries: a stable code, a message template, structured fields, and an optional cause chain.

## 3. Diagnostic contract (compile time)

Each diagnostic has:

- `code`: stable, categorical, never reused (e.g. `TYP002`). Format fixed by [../compiler/SEMANTIC_PATCH_API.md](../compiler/SEMANTIC_PATCH_API.md).
- `severity`: error, warning, info.
- `span`: byte offsets and 1-based line and column. **Line and column are mandatory**; Astra's audit found null positions on several codes, which this rule forbids.
- `expected` / `actual` where meaningful.
- `fix`: optional, with a `safety` class (`format-only`, `behavior-preserving`, `api-changing`, `requires-human-review`). Automatic application is allowed only for the first two, per directive §11.1.
- `explain`: a command that prints the long description for the code.

## 4. Run-time reports

Defects and unhandled errors are reported as JSON on stderr (runtime codes `R....`, separate from compile codes), leaving stdout to the program. Pattern from Astra's separation of runtime codes; the exact schema is an open question.

## 5. Undecidable answers

Verification tools answer `proven`, `refuted` (with counterexample), or `undecidable` (with the reason). Silence is never an answer. This follows the Edict lesson that unreported skipped checks hide coverage gaps.

## 6. Stability

Codes, field names and exit codes are versioned. A change to any of them is a breaking change for agents and needs a migration note.
