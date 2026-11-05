# ADR-0007: Resumability scope

- Status: **Proposed**
- Date: 2026-11-05

## Context

Directive §9.2 asks for a resumability model like Qwik's, and says not to promise universal resumability for arbitrary state. The reference audit found Qwik's docs do not describe how serialized state survives a new deployment.

## Decision

Resumability covers declared state cells and handler ids only ([../app/RESUMABILITY.md](../app/RESUMABILITY.md)). Anything else makes the component non-resumable, and the compiler reports it. Capabilities are never serialized. Deployment skew is handled by a schema version and a defined full reload; this is a proposal to validate, not an established mechanism.

## Alternatives considered

- Full heap serialization: rejected; unsafe with closures, capabilities and external handles, and it cannot be validated.
- Full hydration as the default: rejected; it re-runs the application on load, which the directive wants to avoid.

## Evidence still missing

The deployment-skew experiment in RESUMABILITY §8 has not been run.
