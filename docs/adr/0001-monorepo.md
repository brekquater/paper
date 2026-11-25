# ADR 0001: Keep Paper in a monorepo

- Status: Proposed
- Date: 2026-11-22

## Context

Paper will contain an existing Cargo workspace plus new JavaScript/TypeScript packages and examples. During initial productization, changes across the engine bridge, SDK and React adapter will be tightly coupled.

## Decision

Use one GitHub repository, `Brekquater/paper`. Keep the existing Cargo workspace intact. Add a JS workspace only after M0/M1 establish package boundaries. Publish npm packages independently if/when needed; GitHub repository separation is not required for independent npm releases.

## Consequences

- Cross-layer changes can be reviewed and tested together.
- Avoids early repository and release overhead.
- CI may need path-aware jobs as the monorepo grows.
- Revisit only if a component develops an independent maintainer community, permission boundary or release lifecycle.
