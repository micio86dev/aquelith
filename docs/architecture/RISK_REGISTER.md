# AQELITH Risk Register

Status: **Living document, created at M2.** Likelihood and impact are qualitative (low, medium, high) with the reason. Owner is the project (single contributor for now).

| ID | Risk | Likelihood | Impact | Mitigation | Status |
|---|---|---|---|---|---|
| R1 | Brand or trademark conflict with an existing "Aqu-" software mark | medium | high | Manual registry searches; brand gate; keep private until cleared | open; user decision pending |
| R2 | Graph and text diverge (the Zerolang failure class) | medium | high | ADR-0002; single checker; re-check after format | open; no prototype |
| R3 | Persistent-id scheme collides across agents | low | high | Random 128-bit ids; collision test in CI | open |
| R4 | Capability ergonomics too heavy for application code | medium | medium | Attenuation and defaults; measure in M6 prototype | open |
| R5 | Memory model rejects patterns that users need | medium | medium | Lexical borrows with clear diagnostics; document limits | open |
| R6 | Resumability deployment skew not resolved by the proposed reload | medium | medium | Schema version; experiment in M6 | open |
| R7 | Performance claims made before measurement | medium | high | Methodology; no public claim without raw data (directive §15) | mitigated by process |
| R8 | Competitor evidence ages or is wrong (audited on one day, living projects) | high | medium | Re-audit before M4 and before any public comparison | open |
| R9 | Single-contributor bus factor | high | high | Documentation discipline; ADRs; task logs | open |
| R10 | Limited local hardware (8 GiB, external SSD, Docker VM 4 GiB) skews build and benchmark results | high | medium | Declare hardware; reference machine for budgets | open |
| R11 | Toolchain not pinned; reproducibility weakened | medium | medium | Pin in M4 via `rust-toolchain.toml` (ADR-0001) | open |
| R12 | Docker image cache lost after cleanup (this session removed lab images) | high | low | Rebuild from documented steps; scripts kept in scratchpad, not in repo | accepted |
| R13 | Undetected secret or unsafe default in dependencies | medium | high | Dependency audit, secret scanning in CI | open until CI exists |
| R14 | Web output hides semantics (cloaking, a11y loss) | medium | high | ADR-0006; renderer conformance | open |
| R15 | WASM assumed to be a security boundary it is not | medium | high | Security guarantees list what WASM does not give | mitigated by documentation |
| R16 | Parser crash on hostile input (Zerolang #435 class) | medium | high | Depth limits, fuzzing from day one | open |
| R17 | Docs drift from implementation (Orison and Naze drift observed) | high | medium | Generate facts from CI; status matrix | open |
