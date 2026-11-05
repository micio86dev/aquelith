# Environment Audit (M0)

Date: 2026-11-05. All values below were observed by running the listed commands.

## Machine
| Item | Value |
|---|---|
| OS | macOS 26.5.2 (build 25F84), Darwin 25.5.0 |
| CPU | Apple M1, 8 cores, arm64 |
| RAM | 8 GiB |
| Workspace volume | `<local-disk>` (external, APFS, ~55 GiB free) |
| Docker | 29.4.3, Docker Desktop VM: 8 CPU, ~4 GiB RAM |

## Toolchain
| Tool | Version | Note |
|---|---|---|
| rustc / cargo | 1.89.0 (Homebrew, `/opt/homebrew/bin/rustc`) | Only the `aarch64-apple-darwin` target is installed in its sysroot |
| rustup | 1.29.0 | Installed, but the active `rustc` is the Homebrew one, so rustup does not pin the toolchain |
| Node | 24.18.0 | |
| pnpm / npm | 11.22.0 / 11.16.0 | |
| git / gh | 2.51.0 / 2.98.0 | |
| Claude Code | 2.1.318 | |
| gentle-ai | 4.3.1 | Review mode (RDD): on, decided by global setting |
| gga | 2.10.1 | Provider `claude`, rules file `AGENTS.md`, strict mode on |

## Limitations
1. **Hardware is modest** (8 GiB RAM, 4 GiB for the Docker VM). Heavy competitor builds such as LLVM-based ones may fail or be slow. Benchmarks from this machine are single-machine data points only.
2. **Rust toolchain is not pinned.** Cross targets (`wasm32-*`) need `rustup`-managed rustc. Decide in M2/M3 whether to add a `rust-toolchain.toml`.
3. **Folder name mismatch:** the directory is `aquelith`, the codename is `AQELITH`. Kept as is.
4. **External volume:** I/O-heavy benchmarks on this SSD are not representative of an internal disk.
5. **GGA defaults** only matched `*.ts,*.tsx,*.js,*.jsx`. The project `.gga` now also matches `*.md,*.rs`. GGA reported its pre-commit hook as already installed, so the hook's behavior was not changed by this audit.
6. **No GitHub remote** is configured; none will be created before the brand gate.
7. The `plugin:data:definite` MCP server failed to connect (endpoint not found). It is unrelated to this project.

## Isolation policy
Third-party code lives in `<local-disk>/aqelith-lab/`, outside this repository, and is built and run in Docker.
