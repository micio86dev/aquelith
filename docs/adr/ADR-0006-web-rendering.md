# ADR-0006: Web rendering uses the semantic DOM

- Status: **Accepted as a constraint** from the directive §8.3 and §9.3; the rendering mechanics remain **Proposed**
- Date: 2026-11-05

## Context

The directive forbids Canvas2D as the only web renderer and requires semantic HTML, keyboard navigation, focus management, accessibility labels, form semantics and SEO metadata. The Naze audit found a Canvas2D renderer with a hidden ARIA mirror: the mirror has no event listeners, is rebuilt on every render pass, and the static HTML is hidden with `display:none` once WASM starts. It is the bypass the directive warns about.

## Decision

- The web target renders the view tree as real DOM elements with their semantic roles.
- Canvas may be used for specific elements (charts, media) only when declared in the view tree with a text alternative.
- Static HTML produced for SEO stays visible; it is not hidden when the runtime starts.

## Open questions

- Reconciliation strategy (fine-grained updates vs full re-render of subtrees) is not chosen. A prototype belongs to M6.
- Whether any element type may skip its DOM element. Default: none.

## Evidence

- Naze audit (PI accessibility; DOM mirror observations).
- Directive §9.3 renderer conformance requirements ([../platforms/CONFORMANCE.md](../platforms/CONFORMANCE.md) §4).
