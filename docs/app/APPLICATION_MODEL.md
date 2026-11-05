# AQELITH Application Model

Status: **Proposed (M2).** Describes the intended application layer. Nothing is implemented; M6 is the first milestone that builds it, as a vertical slice.

## 1. Scope (directive §9)

Components, layout, styling, state, derived state, events, forms, validation, routing, data fetching, server functions, authentication integration, streaming, error boundaries, accessibility, localization, animation, testing.

## 2. Core concepts

- **Component:** a function from props and state to a view tree. Declared with an explicit `uses` clause for effects (e.g. `ui`, `net`).
- **State cell:** a value with an identity in the semantic graph. Derived state is a pure function of cells.
- **View:** a tree of semantic elements (`main`, `nav`, `button`, `form`, `input`, `label`...), not an opaque canvas. The mapping from view to DOM is one-to-one for the web target.
- **Event:** an input from the platform, delivered to a handler declared in the component.
- **Route:** a path pattern to a component, with guards that are ordinary functions.
- **Server function:** a function marked for execution on the server; its signature is part of the API surface, and it declares its effects.

## 3. Reactivity

Fine-grained: dependencies are computed at compile time from the graph, so an update re-evaluates only the derived values and view nodes that depend on the changed cell. No generalized virtual DOM by default (directive §9.1). Whether the web target uses signals, compiled updates, or both is an open question resolved by the M3-style prototype in M6.

## 4. Forms and validation

Validation rules are declared on fields and produce typed errors (ERROR_MODEL). The same rule runs on client and server; the server result is authoritative. Client validation is a usability layer, not a security control (directive §6.4).

## 5. Accessibility as a requirement

- Every interactive element has an accessible name by construction, or the compiler reports a diagnostic.
- Focus order follows document order; focus management is part of the routing API.
- The semantic tree is the accessibility tree; no parallel mirror is maintained (contrast with the Naze audit: hidden ARIA mirror, PI).

## 6. Testing

Component tests run without a browser (headless view tree). Browser E2E and accessibility tests run against the generated web output (see [../testing/TEST_STRATEGY.md](../testing/TEST_STRATEGY.md)).

## 7. Non-goals for v1

No native mobile rendering; no WebView-only mobile strategy (directive §8.3); no Canvas2D-only web renderer.
