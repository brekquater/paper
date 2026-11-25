# ADR 0002: Preserve the Rust engine and add a thin SDK layer

- Status: Proposed
- Date: 2026-11-22

## Context

The upstream project has a substantial Rust document model, editing/session engine, layout, formats, renderer and browser/WASM app. Rewriting these components would discard significant existing work and increase compatibility risk.

## Decision

Preserve the Rust engine. Add a TypeScript API and framework adapters around proven browser/WASM capabilities. Any required Rust changes should be narrow, justified and covered by Rust tests. A rewrite or renderer replacement requires a separate ADR and explicit operator approval.

## Consequences

- Reuses mature document logic.
- SDK quality depends on discovering a safe, stable bridge.
- JavaScript↔WASM boundary performance and lifecycle must be measured.
- The public API must not expose unstable internal Rust structures.
