# Conformance report: Linux server target

Status: **partial.** The interface conformance (C2 for loopback HTTP) passes. The language corpus conformance (C1) passes on the corpus in `compiler/tests/corpus/server/`, whose expected outputs await the owner's review. Level definitions: [CONFORMANCE.md](CONFORMANCE.md). Test file: `compiler/tests/conformance_server.rs`.

## Run

```text
cd compiler && cargo test --test conformance_server -- --nocapture
```

Last run (2026-11-06, Apple M1, macOS 26.5.2): interface `loopback-http` passed 4, failed 0; corpus C1 passed on 4 files: 12 functions listed, 11 run, 1 refused for its declared effect.

## Interface: loopback HTTP (C2)

| Case | Requirement | Result |
|---|---|---|
| I1 success path | `GET /` returns 200 and the handler's text, byte for byte | pass |
| I1 localhost name | `Host: localhost:<port>` is accepted | pass |
| I2 typed failure: host | another Host gets 403 before the handler runs | pass |
| I2 typed failure: method | `POST` gets 405 | pass |
| I2 robustness | a refused request does not stop the next one | pass (test `the_server_answers_after_a_refused_request`) |
| I3 unsupported capability | a handler with a declared effect is refused at start; the check runs before the socket is bound | pass (test checks the error variant) |
| I1 query string | the handler sees the path only | pass |

Not covered, and so not claimed: the audit list of denials (CONFORMANCE.md section 3 asks for it; the host does not record denials yet), and the renderer tests (the target is not a UI target).

## Language corpus (C1): met on the server corpus

The corpus is `compiler/tests/corpus/server/`: four programs with zero-argument functions (integers, unsigned integers, booleans, strings, unit, records, calls, field access). Each `.expected` file lists one line per function, in source order. The expected values were derived by hand from the language semantics, not by running the compiler. The test fails when a value differs; a mutation check showed that a wrong expected value fails it.

Open: the owner reviews the `.expected` files before the result counts as conformance. The accepted corpus in `tests/corpus/accept` still has no zero-argument function, so it is not part of C1.

## Reporting rule

A skipped or ignored case is never counted as a pass. The counts above are the only claim this report makes.
