# ADR-0001: Implementation language and toolchain

- Status: **Proposed**, pending prototype evidence (M3 did not test language choice; it is a hypothesis, see §5)
- Date: 2026-11-05

## Context

The directive names Rust as the primary implementation language, "subject to architecture validation". Local toolchain: Homebrew `rustc`/`cargo` 1.89.0 on the host; rustup 1.29.0 is installed but not the active toolchain (see [../reports/ENVIRONMENT_AUDIT.md](../reports/ENVIRONMENT_AUDIT.md)). Competitor compilers are in Rust (Orison, AICore's front end, Naze), in C (Zerolang's native compiler, about 128k lines), and in TypeScript (Edict).

## Decision

Implement the compiler, toolchain and runtime core in Rust. Pin the toolchain with `rust-toolchain.toml` once M4 begins, using a stable channel with an explicit version. Web glue code is TypeScript only where the browser requires it, and no `any` is allowed in it (directive §19).

## Alternatives considered

- C: rejected for the core; Zerolang's audit shows that a C front end needs explicit recursion and size limits, which Rust does not give by default but which are required regardless.
- TypeScript for the whole toolchain: rejected for the compiler core. Inference, not measured: Edict's audit lists a pure-JS WASM interpreter and a built-in solver written so the compiler runs inside QuickJS, which suggests portability pressure shapes the design. No performance comparison was made.

## Consequences

- Build times and memory use of Rust builds must be measured on the reference machine (a 4 GiB Docker VM was too small for some competitor Rust builds in M1; see audit logs).
- Fuzzing and sanitizers are available for Rust code.

## Evidence still missing

No prototype compared implementation languages. Prototypes P1 and P2 are Rust but test architecture ideas, not language performance.
