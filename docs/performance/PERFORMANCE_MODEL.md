# AQELITH Performance Model

Status: **Proposed (M2).** No AQELITH performance number exists. This document defines what is measured, so that later work measures the same things.

## 1. Principle

Performance is measured across the whole lifecycle, not only the compiler or only the runtime (directive §7). Competitor numbers are the project's own unless stated; this model does not reuse them as targets.

## 2. Compiler (directive §7.1)

Cold start, warm start, parsing, type checking, graph generation, full build, incremental build, dependency invalidation, optimization, code generation, peak memory, cache effectiveness.

## 3. Runtime (directive §7.2)

Startup, CPU, memory footprint, allocation count, garbage-collection overhead (N/A for the core model, see [../language/MEMORY_MODEL.md](../language/MEMORY_MODEL.md)), throughput, latency, tail latency, concurrency, network overhead, rendering cost, battery impact on mobile.

## 4. Development experience (directive §7.3)

Save-to-feedback time, hot-reload latency, state preservation, test execution time, diagnostics generation time, agent repair cycles (number of attempts to a passing build).

## 5. Backend strategy (directive §7.4)

Evaluated options: JavaScript generation for browser integration; WebAssembly for suitable compute workloads only; native machine code; interpreter or IR execution during development; AOT for production; JIT only if measured to help.

Decision for M2 ([../adr/ADR-0005-backend-strategy.md](../adr/ADR-0005-backend-strategy.md)): IR interpreter for development first, then one production backend chosen by measurement. No LLVM, Cranelift or JIT in the first implementation.

## 6. What "faster" may mean here

Only a measured comparison on the same workload, hardware, build mode and input, with the methodology in [BENCHMARK_METHODOLOGY.md](BENCHMARK_METHODOLOGY.md). Anything else is not published.
