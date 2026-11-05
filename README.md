# AQELITH (provisional name)

Research and architecture phase for a universal application language designed for humans and AI agents.

> The name AQELITH is an internal working codename. It has not been cleared legally or commercially. Do not publish packages or releases under it.

Status: see [docs/PROJECT_STATUS.md](docs/PROJECT_STATUS.md).

Implemented so far: a v0 compiler subset in `compiler/` (lexer, parser, checker, formatter, `check`/`fmt`/`query`/`explain` CLI). Try it:

```text
cd compiler && cargo test && cargo run -q -- check tests/corpus/accept/arith.aq
```

Not implemented: web runtime, native backends, patch API, MCP server, capabilities at run time. The v0 language is described in [docs/language/GRAMMAR.md](docs/language/GRAMMAR.md) and [skills/aqelith/SKILL.md](skills/aqelith/SKILL.md).
