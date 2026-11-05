# AQELITH Resumability Model

Status: **Proposed (M2).** Directive §9.2 requires this document to say what is serialized, how it is validated, how closures and handlers are represented, how deployment skew is handled, and what cannot be serialized. Several points remain open; they are marked.

## 1. Goal

Serve a rendered page whose interactive state can be resumed on the client without re-running the whole application on load, so that only the code for an interaction is fetched when the interaction happens.

## 2. What is serialized

- Component state cells that the server rendered, as typed data, using a schema generated from the state declaration.
- The identity of the handler each event triggers (a stable handler id), not its code.
- Route and locale parameters.

## 3. What cannot be serialized

- Closures that capture non-serializable state (open files, sockets, capability values).
- Capability values: they are never serialized; they are re-granted on resume ([../security/CAPABILITY_MODEL.md](../security/CAPABILITY_MODEL.md)).
- Arbitrary program heap. The directive forbids promising universal resumability; this model covers only the declared state cells and handler ids.
- Values of types that do not declare a serialization schema. Such a value makes the component non-resumable, and the compiler says so.

## 4. Validation

- Deserialization is typed: the schema is checked before any state is constructed. A mismatch is rejected; no fallback to untyped data.
- Handler ids are checked against the handler table of the deployed build; an unknown id is rejected.
- No executable code travels in the serialized state. This is the test that catches the escaping class of bug seen in the Naze audit (`</script>` in hydration JSON).

## 5. Deployment skew (open question)

When a new version is deployed while a client holds state from the old version, the handler id or state schema may no longer exist. Qwik's docs, reviewed in [../research/REFERENCE_ECOSYSTEMS.md](../research/REFERENCE_ECOSYSTEMS.md), do not describe a migration mechanism; that review recorded it as a gap. Proposal for v1: each build carries a schema version; a mismatch triggers a full reload with the state discarded and reported. This is a proposal, not a verified design.

## 6. Closures and handlers

A handler is named by a stable id derived from the component's persistent id and the handler's position in its declaration (position is acceptable here because the table is per build and versioned). Its code is loaded on first use. Handlers must be pure functions of state plus event data plus capabilities supplied by the runtime.

## 7. Security

The state payload is data, not code. Serialization output is HTML-escaped. Resumption never executes server-side functions directly; it calls declared server functions through the normal typed interface.

## 8. Tests required before this is marked implemented

- Round trip of each supported state type.
- Rejection of a payload with an unknown handler id.
- Rejection of a payload with a mismatched schema version.
- Escaping test with `</script>` and `<!--` inside string state.
- Deployment-skew experiment: old state resumed against a new build yields the defined reload behavior.
