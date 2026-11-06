# Conformance report: Linux server target

Status: **partial.** The interface conformance (C2 for loopback HTTP) passes. The language corpus conformance (C1) is not met. Level definitions: [CONFORMANCE.md](CONFORMANCE.md). Test file: `compiler/tests/conformance_server.rs`.

## Run

```text
cd compiler && cargo test --test conformance_server -- --nocapture
```

Last run (2026-11-06, commit of this report, Apple M1, macOS 26.5.2): interface `loopback-http` passed 4, failed 0. The corpus test is marked `ignore` and counted as skipped, not as a pass.

## Interface: loopback HTTP (C2)

| Case | Requirement | Result |
|---|---|---|
| I1 success path | `GET /` returns 200 and the handler's text, byte for byte | pass |
| I1 localhost name | `Host: localhost:<port>` is accepted | pass |
| I2 typed failure: host | another Host gets 403 before the handler runs | pass |
| I2 typed failure: method | `POST` gets 405 | pass |
| I2 robustness | a refused request does not stop the next one | pass (test `the_server_answers_after_a_refused_request`) |
| I3 unsupported capability | a handler with a declared effect is refused before the socket opens | pass |
| I1 query string | the handler sees the path only | pass |

Not covered, and so not claimed: the audit list of denials (CONFORMANCE.md section 3 asks for it; the host does not record denials yet), and the renderer tests (the target is not a UI target).

## Language corpus (C1): not met

The accepted corpus has no zero-argument pure function that can run. Its functions take parameters or declare effects, and the interpreter runs only the former when given arguments. The corpus test is `#[ignore]`d for this reason. C1 needs a corpus with expected outputs for the server target; it is not written yet.

## Reporting rule

A skipped or ignored case is never counted as a pass. The counts above are the only claim this report makes.
