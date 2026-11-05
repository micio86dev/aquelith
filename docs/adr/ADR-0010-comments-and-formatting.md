# ADR-0010: Comments and canonical formatting (answers Q10)

- Status: **Accepted for v0** (decided by the project owner delegating the choice, 2026-11-05)
- Date: 2026-11-05

## Context

Q10 asked whether the formatter round trip is lossless for all semantic data. Comments are the main thing a formatter can lose. The directive also requires one canonical form per concept.

## Decision for v0

- **No comments in v0.** `//` is rejected with LEX003. This keeps the formatter lossless by construction: the text is a pure function of the syntax tree.
- Canonical form: 4-space indentation, one statement per line, one blank line between items, parentheses only where precedence requires them, no trailing spaces, exactly one trailing newline.
- The formatter is syntax-only: it formats any text that parses, including ill-typed text, and never changes meaning.
- Required property: `format(parse(format(x))) == format(x)` for every accepted program (tested over the corpus).

## Consequences

- Comments come back in a later version with a defined attachment rule (attached to the following item, preserved verbatim). Until then documentation lives in `docs/`.
- Agents do not need a comment-stripping step.

## Evidence

Zerolang's text is a projection of the graph and it needed a sync layer; keeping the formatter a pure function of the tree is the simplest way to avoid that divergence (see ADR-0002).
