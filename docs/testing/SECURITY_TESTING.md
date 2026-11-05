# AQELITH Security Testing

Status: **Proposed (M2).** Each row maps a mechanism from [../security/](../security/) to the test that would fail if the mechanism were removed.

## 1. Test matrix

| Mechanism | Test that must fail if the mechanism is removed | Layer |
|---|---|---|
| Effect checking | Missing-effect fixture accepted (must be rejected) | Compile |
| Use-after-move | Use-after-move fixture accepted | Compile |
| Capability values | Forged-capability fixture compiles (must not) | Compile |
| Patch preconditions | Stale patch applied (must be rejected) | Compiler API |
| Patch validation | Type-invalid patch written (must be rejected) | Compiler API |
| Stale-write atomicity | Store changed after rejected patch | Compiler API |
| Host-call denial | Call without capability performs the effect | Runtime |
| Path confinement | `../` escape reaches a file outside the root | Runtime |
| Real-path checks | Symlink escape reaches a file outside the root | Runtime |
| Network default-deny | Undeclared host reached | Runtime |
| Environment scrubbing | Secret variable visible in child process | Runtime |
| HTML escaping | Script injected through a state value executes | Web |
| Resumption payload validation | Unknown handler id accepted | Web |
| Hydration escaping | `</script>` in state breaks out of the script element | Web |
| SQL parameter binding | String-built query accepted | Library |
| Dependency capability diff | Added `net` effect not reported | Package |
| Deserializer robustness | Malformed IR or patch crashes instead of erroring | Fuzz |
| Parser depth limit | Deeply nested input crashes the compiler (Zerolang #435 class) | Fuzz and regression |

## 2. Required techniques

- Fuzzing of: lexer, parser, IR deserializer, patch parser, resumption payload parser, runtime codecs. Fuzzing runs in CI with a fixed time budget and a corpus kept in the repository.
- Sanitizers (AddressSanitizer, UndefinedBehaviorSanitizer) on the runtime and any native code.
- Dependency audit (vulnerabilities, licenses) and secret scanning in CI (directive §19).

## 3. Rules

- Security tests run in CI on every merge.
- A security finding is either fixed with a regression test, or recorded in the risk register with an owner. Silent acceptance is not allowed.
- Results are attached to the milestone report; no "secure" statement without them ([../security/SECURITY_GUARANTEES.md](../security/SECURITY_GUARANTEES.md) §5).

## 4. Out of scope for now

Penetration testing of deployed applications; third-party audit (requires a decision on external spend, a gate in the directive).
