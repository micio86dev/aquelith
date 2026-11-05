# AQELITH Threat Model

Status: **Proposed (M2).** Scope: the language, compiler, package system, runtime and generated web applications, as designed. Each threat lists a mitigation status: **designed** (in the M2 docs), **tested** (has a test plan), **unmitigated** (known gap).

## 1. Assets

Source code, build artifacts, package registry metadata, user data handled by applications, credentials and secrets used by applications, the compiler toolchain, and agent-issued patches.

## 2. Actors

| Actor | Capability assumed |
|---|---|
| Malicious dependency author | Publishes code that runs at build or run time |
| Compromised build tool or CI runner | Alters compiler output |
| Malicious or mistaken agent | Produces source patches at machine speed |
| Remote attacker against a deployed app | Sends crafted HTTP, WebSocket or form input |
| Malicious web page or browser extension | Runs in the same origin as a generated web app |
| Local user with access to the device | Reads files and environment |

## 3. Threats

| # | Threat | Affects | Mitigation designed | Status |
|---|---|---|---|---|
| T1 | Malicious dependency reads files or the network | Packages | Capability manifest; dependency capability diff in audit (Orison audit: AUD0001/2 pattern) | designed |
| T2 | Compromised build tool alters output | Toolchain | Reproducible builds; hermetic manifest; signed provenance (AICore audit: HMAC provenance, IE) | designed; signing scheme open |
| T3 | Agent produces an unsafe or stale patch | Patch API | Preconditions on graph hash and node values; validation before write; one authoritative verdict ([../compiler/SEMANTIC_PATCH_API.md](../compiler/SEMANTIC_PATCH_API.md)) | designed; prototype P1 |
| T4 | Unsafe FFI corrupts memory | Runtime | FFI is an explicit trusted boundary; memory passing rules ([../language/MEMORY_MODEL.md](../language/MEMORY_MODEL.md)) | designed |
| T5 | Memory corruption in the runtime | Runtime | Sanitizers in CI; fuzzing of parser and runtime ([../testing/SECURITY_TESTING.md](../testing/SECURITY_TESTING.md)) | designed; no runtime yet |
| T6 | Injection (SQL, shell, template) | Apps | Typed query interfaces with parameter binding (Naze audit: parameterized `$N` placeholders IT); no string-built queries in the standard library | designed |
| T7 | Cross-site scripting | Web apps | Escaping by default in the view layer; no raw HTML sink without an explicit, effect-marked escape hatch | designed |
| T8 | Server-side request forgery | Backends | Outbound HTTP requires the `net` capability with an explicit host allow-list; default deny | designed |
| T9 | Path traversal | Backends | `fs` capability scoped to a root; path canonicalization with real-path checks (Edict audit: lexical-only check was a gap) | designed |
| T10 | Secret leakage through logs or child processes | Apps, toolchain | Secrets are a type that cannot be printed; child processes receive scrubbed environments by default (Edict audit: full environment inheritance was a gap) | designed |
| T11 | Tenant isolation failure | Multi-tenant apps | Tenant context is a capability value, not a global; cross-tenant access requires an explicit grant | designed; no test yet |
| T12 | Race conditions | Runtime | Structured concurrency and ownership rules ([../language/CONCURRENCY_MODEL.md](../language/CONCURRENCY_MODEL.md)); data-race checking is an open question | partly designed |
| T13 | Unsafe deserialization | Apps | Deserialization into typed values only; no type-tag code execution | designed |
| T14 | Sandbox escape of the runtime | Runtime | Capabilities enforced at runtime, OS-level sandboxing where available; no claim beyond what is tested ([SECURITY_GUARANTEES.md](SECURITY_GUARANTEES.md)) | partly designed |
| T15 | Malicious plugin | Toolchain | Plugins are packages; same capability rules as T1 | designed |
| T16 | Browser origin boundary bypass | Web apps | Generated apps follow same-origin rules; no cross-origin capability passing without an explicit protocol | designed |
| T17 | Supply-chain compromise through the registry | Packages | Signed or checksummed packages; the Orison audit found non-cryptographic checksums, so this is a required design change | unmitigated until signing is designed |
| T18 | Parser or compiler crash on hostile input | Toolchain | Depth and size limits; fuzzing from day one. The Zerolang audit reproduced a SIGSEGV on deep input (issue #435) | designed; reproduced in a competitor |

## 4. Out of scope

- Physical attacks on devices.
- Attacks on operating systems or browsers themselves, beyond the assumptions in [SECURITY_GUARANTEES.md](SECURITY_GUARANTEES.md).
- Authorization decisions of backend business logic: AQELITH provides the mechanisms; the application decides who may do what.

## 5. Evidence base

Competitor audits in [../research/competitors/](../research/competitors/). Each row above that cites a competitor records a gap observed there, not a claim about that project's overall security.
