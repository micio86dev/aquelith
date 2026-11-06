# Threat model: Linux server target (loopback HTTP host)

Status: **written for the minimal host (S3).** Covers `compiler/src/serve.rs` and `compiler/src/interp.rs` as used by `aqelith-serve`. It does not cover a production deployment: none exists.

## Assets

- The host process and its CPU time.
- The handler program (trusted code supplied by its author; see actors).
- Any local file the host can read or write. The host reads only the program file given on its command line and writes nothing.

## Actors and trust boundaries

| Actor | Trust | Can reach the host through |
|---|---|---|
| Handler author | Trusted (writes the program) | The program file |
| Local process on the same machine | Untrusted | 127.0.0.1 |
| Web page in a browser on the same machine | Untrusted (DNS rebinding) | 127.0.0.1 with a foreign Host name |
| Remote machine | Not reachable by design | Nothing: the socket is bound to 127.0.0.1 only |

## Threats and mitigations

| # | Threat | Mitigation | Test |
|---|---|---|---|
| T1 | Remote access to the host | Bind to 127.0.0.1 only | `serve.rs` `start` (no other bind path) |
| T2 | DNS rebinding from a browser | Host must be `127.0.0.1:<port>` or `localhost:<port>`; else 403 before the handler runs | `foreign_host_names_are_refused_before_the_handler_runs`, conformance I2 |
| T3 | Oversized request head to exhaust memory | At most 8 KiB, else 431 | `an_oversized_head_is_refused_and_the_server_keeps_serving` |
| T4 | Slow client holding the single connection | 5 s deadline for the whole head, per-read timeout | Covered by code (`read_head`); no automated test yet |
| T5 | Method other than GET used to change state | Only GET is served; others 405. The handler has no way to write files or sockets | `only_get_is_served` |
| T6 | Handler with effects reaching the network from inside the language | `net` and `io` are refused by the interpreter; a handler that declares an effect is not started | `a_handler_that_declares_an_effect_is_not_started`, `EffectNotAvailable` tests |
| T7 | Handler that runs forever or uses exponential work | Step budget of 1,000,000 instructions and depth limit 256 | `exponential_work_at_low_depth_stops_at_the_step_budget` |
| T8 | A panic in one request stops the server | `catch_unwind` around each request | Code only; not yet exercised by a test |
| T9 | Information leak through errors | A failing handler answers 500 with no body (not reachable in the current subset) | Code only |
| T10 | Security headers absent | `Content-Security-Policy: default-src 'none'`, `nosniff`, `no-store` on every answer | `a_get_returns_the_handler_text_with_security_headers` |
| T11 | Path traversal | The handler receives a string and cannot open files; the host does not map paths to files | By construction: the host has no file access per request |
| T12 | Request smuggling through a second Host header | Two Host headers give no host and are refused | `malformed_requests_are_bad_requests` (in the web crate's dev server tests; the same rule is in `parse_head`) |

## Residual risks (accepted, documented)

- **No TLS, no authentication.** Acceptable only because the socket is loopback-only. Exposing it needs a new threat model.
- **One connection at a time.** A slow but valid client delays others for up to 5 seconds per request (T4).
- **The handler is trusted code.** A malicious handler author has the same power as the program author; the language limits (no file or network effects) bound what it can do, not what it can compute within the budget.
- **The step budget is a constant, not a measured limit.** It bounds work; it does not promise a time.
- **Not audited by a third party.** This document is the author's analysis.

## Not yet done

- Automated tests for T4 and T8.
- An audit list of denials (the conformance report lists it as missing).
