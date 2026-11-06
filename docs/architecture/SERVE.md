# Loopback HTTP host (milestone S3)

Status: **implemented for the minimal handler.** Source: `compiler/src/serve.rs`, binary `compiler/src/bin/aqelith-serve.rs`. Tests: `compiler/tests/serve.rs` (7 tests, real sockets on 127.0.0.1).

## What it does

```text
aqelith-serve site.aq --port 8080
curl http://127.0.0.1:8080/docs    # 200, body = what handle returns for "/docs"
```

The program provides one function: `fn handle(path: str) -> str`, with no declared effect. For each request the host runs the handler with the request path (query string removed) and sends the returned text as the body.

## Decisions taken (reversible)

| Decision | Chosen | Why |
|---|---|---|
| Who owns the socket | **The host**, not the language. The `net` effect stays refused inside the language. | Keeps the language free of I/O and makes every socket visible in one module. |
| Request and response values | **Minimal:** the request is its path (`str`); the response is its body (`str`). | v0 has no record types. Headers, methods and status codes need records, which are a separate slice. |
| Binding | **127.0.0.1 only.** | No network exposure from a development server. |
| Host header | Must be `127.0.0.1:<port>` or `localhost:<port>`, otherwise `403`, before the handler runs. | Blocks DNS rebinding from a browser. |
| Methods | `GET` only; other methods `405`. | The handler has no request body to read. |
| Head size and time | At most 8 KiB and 5 seconds, otherwise `431` or `408`. | Bounds memory and a slow client's hold on the server. |
| Concurrency | One connection at a time. | Simple and bounded; no shared state. |
| Faults | A panic while serving one request drops that connection; the accept loop keeps running. A run that exceeds the step budget answers `500`. | One bad request must not stop the server. |
| Response headers | `Content-Security-Policy: default-src 'none'`, `X-Content-Type-Options: nosniff`, `Cache-Control: no-store`. | Text responses cannot run or embed anything. |

## Limits

- The handler cannot read a request body, headers or the method: they are not passed in.
- A handler that fails at run time would answer `500` with no detail. The v0 subset has no run-time error for a `str` handler, so this branch is not reachable yet; it is kept as the defensive answer for later slices. A handler that exceeds the step budget also gets `500`, but that path is not yet tested end to end.
- A handler that declares an effect (for example `uses net`) is refused at start, not at the first request.
- No TLS, no keep-alive, no compression. Loopback only.
- No performance claim: one connection at a time is a correctness choice, not a measured limit ([../performance/BASELINE.md](../performance/BASELINE.md)).
