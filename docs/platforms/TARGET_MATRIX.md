# AQELITH Target Matrix

Status: **Proposed (M2). Every target below is "planned" unless stated otherwise.** The matrix is the authoritative place where support status is recorded. A cell may move to "supported" only when the criteria in [PLATFORM_STRATEGY.md](PLATFORM_STRATEGY.md) §3 are met and the conformance run is recorded.

## 1. Operating systems

| OS | Stage | Status | Notes |
|---|---|---|---|
| Browser environments | 1 | planned | First target |
| Linux (x86_64, ARM64) | 2 | planned | Server target |
| macOS (ARM64, x86_64) | 3 | planned | Development host for this project (Apple Silicon) |
| Windows (x86_64) | 3 | planned | |
| iOS | 4 | planned | Native UI; no WebView-only path |
| Android | 4 | planned | Native UI; the Naze WebView wrapper is not the baseline |
| Embedded RTOS | 5 | planned | Feasibility not assessed; reference audit found Rust `no_std` only |

## 2. CPU architectures

| Architecture | Status | Notes |
|---|---|---|
| x86_64 | planned | |
| ARM64 | planned | The Zerolang audit observed a direct AArch64 backend rejecting a conformance fixture; AQELITH must not assume a backend is complete for every architecture it targets |
| ARM32 | planned | Only where a target requires it |
| RISC-V | planned | Through backend interfaces; no commitment |

## 3. Execution environments

| Environment | Status | Notes |
|---|---|---|
| Browser | planned (stage 1) | |
| Server | planned (stage 2) | |
| Desktop | planned (stage 3) | |
| Mobile | planned (stage 4) | |
| Edge | planned | Depends on server target |
| Embedded | planned (stage 5) | No-heap profile required |

## 4. Recorded evidence

No conformance run has been executed for any AQELITH target, because none exists. The competitor audits are evidence about those projects, not about AQELITH's targets.
