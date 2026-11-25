# Paper Roadmap

This roadmap is milestone-based, not calendar-based. A milestone closes only when its exit criteria are met. Do not estimate delivery based on AI token usage.

## M0 — Fork baseline and technical audit

**Goal:** understand what the inherited engine and browser app can actually do today.

Tasks:
- Record upstream commit, fork commit, branch and remotes.
- Run untouched build and CI commands; record environment and failures.
- Map the Rust crate architecture and public engine/session interfaces.
- Trace the inherited web app from startup through user input, command dispatch, rendering and file I/O.
- Identify the browser control protocol and any host-application bridge.
- Inspect HTML, DOCX, PDF, JSON and legacy `.doc` reader/writer paths.
- Inspect comments, revisions, author metadata and compare/combine implementation.
- Determine what native serialization, if any, is already supported.
- Create a representative document test corpus and browser test plan.

**Exit gate:** a source-backed baseline audit identifies a minimal vertical slice. No broad rename or package creation before this gate.

## M1 — Browser embedding proof of concept

**Goal:** prove the existing browser editor/engine can be hosted and controlled from a separate web application without rewriting it.

Tasks:
- Build the existing WASM/browser app unchanged.
- Test iframe/host integration, direct module integration or the existing control protocol; choose based on evidence.
- Demonstrate initialization, readiness, command invocation, event reception, document load, document save/export and disposal.
- Measure cross-boundary call frequency and identify hot paths.
- Record browser, CSP, worker, font and asset-loading constraints.

**Exit gate:** a separate minimal host page can initialize, edit, save/export and destroy the Paper instance reliably. Chosen integration mechanism has an ADR.

## M2 — Framework-independent TypeScript core

**Goal:** provide a typed, framework-neutral lifecycle and document API.

Potential responsibilities:
- Load WASM/assets lazily and configure paths.
- Create/dispose editor sessions.
- Load/import supported documents.
- Execute commands and query capabilities.
- Subscribe/unsubscribe to events.
- Export supported formats as text or binary values.
- Surface structured errors and partial-support warnings.

**Exit gate:** unit/type tests and a vanilla browser example demonstrate the public core API. No framework-specific logic in core.

## M3 — Shared editor UI and React adapter

**Goal:** produce the first usable embeddable product.

Tasks:
- Determine whether existing egui/canvas UI can be embedded and configured adequately.
- Implement the minimum host integration surface.
- Add Basic/Advanced presets and custom toolbar configuration only where command support is verified.
- Separate UI visibility from command permissions.
- Support parent-container sizing and a reliable fullscreen mode.
- Provide React component/hooks as thin adapters.

**Exit gate:** React demo supports repeated mount/unmount, controlled document lifecycle and a documented config surface.

## M4 — Document workflows and quality

**Goal:** validate the highest-value document capabilities.

Tasks:
- DOCX upload/import and DOCX export.
- HTML set/get.
- Clipboard paste tests using Word-originated content in supported browsers.
- Comments, author attribution, replies and tracked changes, as supported.
- Compare two host-supplied versions, if existing compare engine can be exposed reliably.
- Native persistence format investigation and decision.
- Large/corrupt file handling and performance measurements.

**Exit gate:** regression corpus, fidelity reports, browser tests and explicit support matrix.

## M5 — Public alpha

**Goal:** make the project usable by external developers.

Tasks:
- Docs, quick start, API reference and working examples.
- Packaging and reproducible local builds.
- CI for Rust, WASM, TypeScript, React and browser tests.
- Version compatibility rules between packages and WASM assets.
- Accessibility and CSP/browser limitation documentation.
- Release checklist and rollback plan.

**Exit gate:** alpha criteria in `PRD.md` pass. Publishing is an explicit operator-approved action.

## Later milestones (not current scope)

- Vue and Angular adapters after the common API stabilizes.
- Plugin API after lifecycle, command and extension contracts are stable.
- Real-time collaboration only after separate architecture and consistency design.
- Enterprise licensing, branding tiers and support after product validation.
- Hosted services only if customers demonstrate a need.

## Current focus

M0 and M1 are done (2026-11-25): the baseline was audited, and the browser component (`apps/paper-web`) mounts, edits, loads, exports, raises events and is destroyed on a host page (ADR 0003). Next is M2: the typed, framework-independent core API. Do not create empty npm packages.
