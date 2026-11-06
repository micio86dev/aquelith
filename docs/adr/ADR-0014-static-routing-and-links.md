# ADR-0014: Static routing and same-site links

- Status: **Accepted for the M6 routing slice** (decided by the project owner, who delegated design choices, 2026-11-06)
- Date: 2026-11-06
- Amends: ADR-0012 (adds a link node and a site build; the runtime is unchanged)

## Context

M6 lists routing. The runtime of ADR-0012 is a single page with declarative events. Client-side navigation needs a router and state across pages, which are not built.

## Decision

1. **Routes are static.** A site is a list of routes, each rendering one page. `/` becomes `index.html`; `/about` becomes `about/index.html`. Each file is a complete page built by the same `render_page` rules.
2. **Links are a view node** `Link { label, href }`, rendered as `<a href>`. The href must be a same-site path: it starts with one `/`, is not `//`, has no backslash, control character or dot segment. External URLs, schemes such as `javascript:`, and protocol-relative URLs are refused.
3. **Links must resolve.** The site build refuses a link to a route the site does not have (`BrokenLink`).
4. **Route syntax is strict.** A route is `/` or a path with no trailing slash, query, fragment or dot segment, and each route appears once.

## Alternatives considered

- Client-side routing with history API: deferred. It needs a state model across navigations, which ADR-0007 limits to declared state, and a runtime change that is not designed yet.
- Allowing external links: rejected for the slice. It needs a policy for outbound navigation that is not decided.

## Consequences

- No state survives a navigation. A form value on one page does not carry to another.
- The build writes static files; a host must serve `about/index.html` for `/about`.

## Evidence

- Automated: `web/tests/site.rs` (6 tests) checks file names, rendered links, broken links, rejected hrefs (external, protocol-relative, javascript, dot segments, backslash), route syntax and duplicates, and empty labels.
- Not done: serving the built files in a browser, and a browser test of navigation.
