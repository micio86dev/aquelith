# ADR-0015: String literals and target-neutral page items

Status: **Accepted for the local experiment** (2026-11-06). Not a release decision.

## Context

The v0 language has no string literals. A landing page therefore cannot be written in AQELITH source: the web target builds pages from Rust values (`counter_page()`, `form_page()`). The owner chose to extend the language so that pages are written in `.aq` files.

The owner also requires that the language and the system stay universal: the core language must not depend on any one output target. A page is a description of content, not HTML.

## Decision

1. **String literals.** A new token `"..."`, read as text by page items. It has no type in the checker. Content is printable ASCII, excluding `"` and `\` (no escape sequences in this slice). Newlines, control characters and bytes above 0x7e are rejected (LEX004 for an unterminated literal, LEX005 for a character not allowed). Strings appear only in page items and are not values of functions in this slice.

2. **Page items.** A top-level `page` item with a route and a fixed set of content statements:

   ```
   page home "/" {
       title "AQELITH";
       h1 "Build software faster";
       p "Text of one paragraph.";
       link "Read more" "/docs";
   }
   ```

   The statements are `title`, `h1`, `h2`, `p`, `link`. The set is closed; a new statement needs an ADR. `title` appears exactly once. `link` takes a label and a same-site route.

3. **Neutral core.** The compiler crate parses and checks pages into a target-neutral tree (`ast::Page`, `ast::PageItem`). It contains no HTML, no tags, no scripts. The checks are about content: non-empty text, a valid route, a unique route, a title present, and links that name an existing route of the module.

4. **Backends.** A backend turns the neutral page tree into an output. The web backend lives in `web/` and depends on the compiler crate. It renders the tree with the existing HTML renderer, so it is one of possibly many backends. A second backend is not built in this slice, so universality is a design constraint here, not yet a demonstrated property.

5. **Effects.** Page items declare no effects and call no functions in this slice.

## Consequences

- `fn` items and `page` items may appear in one module. Function-level tooling (patch, ids, repair, query) keeps working on functions and ignores pages.
- The formatter prints pages canonically, so the fixed-point rule holds for pages too.
- The grammar document (`docs/language/GRAMMAR.md`) and the agent skill (`skills/aqelith/SKILL.md`) are updated to describe only what is implemented.
- Not in this slice: escapes, string values in functions, `str` parameters, images, lists, attributes or styles, client-side behaviour in pages, and a second backend.

## Alternatives considered

- **HTML inside `.aq`.** Rejected: it ties the language to one target and violates the universality constraint.
- **Pages as functions returning view values.** Rejected for this slice: it needs a view type and effects for building trees, which is a larger design.
