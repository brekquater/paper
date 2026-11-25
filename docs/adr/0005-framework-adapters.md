# ADR 0005: One framework-independent core with thin adapters

- Status: Proposed
- Date: 2026-11-22

## Context

The product should support React first and Vue/Angular later without forking document behavior.

## Decision

Business logic, runtime lifecycle, command mapping, event semantics and document I/O live in a framework-independent TypeScript core. Framework packages only adapt lifecycle, props/events and idiomatic framework APIs. React is the first adapter. Create packages only when they contain a real, independently testable boundary.

## Consequences

- Consistent behavior across frameworks.
- Core remains usable from vanilla JavaScript.
- Framework adapters can version independently at npm without splitting Git repositories.
