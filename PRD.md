# Product Requirements Document — Paper

- **Product:** Paper
- **Organization:** Brekquater
- **Repository:** `Brekquater/paper`
- **Status:** Draft for technical validation
- **Initial target:** Browser-based embeddable editor, React integration first
- **Foundation:** Existing Rust document engine, preserved rather than rewritten

## 1. Product vision

Paper is an embeddable document editor and document-engine SDK for web applications. It builds on an existing Rust document engine and exposes its editing, layout and document-format capabilities through a clean JavaScript/TypeScript API.

The product should let a developer integrate a professional document-editing experience into an existing application without operating Paper's cloud, adopting a particular database or rewriting their application architecture.

## 2. Problem

Many web applications need more than a conventional rich-text field. They need Word-compatible documents, page-aware layout, DOCX import/export, review workflows and eventually comparison of saved versions. Building these capabilities independently is expensive and creates document-fidelity problems.

Paper aims to provide the document engine and an embeddable UI while leaving authentication, persistence, business metadata and application workflows under the host application's control.

## 3. Target users and use cases

### Primary users

- Frontend developers embedding an editor in React applications.
- Product teams building document-heavy SaaS products.
- Developers replacing or supplementing a conventional rich-text editor.

### Representative application domains

- Contract and legal document management.
- Banking and credit review workflows.
- Insurance underwriting and claims.
- Enterprise reports, proposals and approvals.
- Document management and content platforms.

These are target use cases, not claims of current production readiness or compliance certification.

## 4. Product principles

1. **Reuse before rewrite:** retain the Rust document model, editing engine, layout, rendering and format code wherever feasible.
2. **Host application owns data:** Paper does not prescribe a database, storage vendor, user system or document metadata schema.
3. **One core, thin adapters:** React, Vue and Angular must share one framework-independent SDK.
4. **Progressive capability:** simple defaults, configurable advanced controls.
5. **Format honesty:** HTML, DOCX and any native format have different fidelity and compatibility properties; communicate them clearly.
6. **No required cloud:** core editing must not require Paper-operated servers or transmit documents externally.
7. **Reliability before breadth:** preserve content, fail clearly, and test real documents before adding a large feature surface.

## 5. Scope for the first public alpha

### P0 — required for an alpha

- A host web application can initialize the Paper editor in a supplied container.
- A framework-independent TypeScript API owns editor initialization, document lifecycle, commands, events, errors and disposal.
- A React adapter exposes the same core capabilities without duplicating editor logic.
- The implementation reuses the existing browser/WASM path wherever the audit demonstrates it is viable.
- A developer can load supported HTML content and retrieve edited HTML.
- A developer can import a DOCX document, edit it, and export a DOCX document, subject to explicitly documented compatibility limits.
- A developer can configure basic toolbar visibility and editor layout.
- The host application controls storage by receiving an output such as `Blob`, `ArrayBuffer` or `Uint8Array` and deciding where to persist it.
- Import, export, initialization and runtime failures are surfaced through typed errors/events; no silent success on partial failure.
- A demo React application proves the end-to-end workflow.

### P1 — target after the vertical slice is stable

- Basic and Advanced presets.
- Fine-grained toolbar item visibility.
- Separate visibility and operation permissions; hiding a button must not be mistaken for disabling the command.
- Embedded, fullscreen and constrained-container layouts.
- File upload/download helpers.
- Clipboard paste fidelity investigation and supported-browser test matrix.
- Expose existing comments, replies, author attribution and tracked changes through stable APIs where the underlying engine supports them.
- Compare two caller-supplied document versions using existing engine comparison functionality, if the audit confirms the current interface is reusable.

### Later, not required for initial alpha

- Vue and Angular adapters.
- Plugin ecosystem.
- Real-time collaboration, OT or CRDT.
- Hosted cloud, document storage or backend service.
- Licensing, paid editions, watermark enforcement or enterprise packaging.
- Template marketplace, AI writing, cloud conversion service.
- Full parity with Microsoft Word or CKEditor.

## 6. Core user stories

### US-01: Embed the editor
As a React developer, I can mount the editor in a component or container, receive a ready event, use its API and dispose it cleanly.

**Acceptance:** a sample React application can mount, edit and unmount repeatedly without leaked workers, event listeners or duplicated WASM instances beyond the documented lifecycle model.

### US-02: Load and retrieve HTML
As a developer, I can provide HTML and retrieve edited HTML.

**Acceptance:** supported HTML structures round-trip through fixtures; unsupported structures are documented and do not silently disappear.

### US-03: Import and export DOCX
As a developer, I can pass a DOCX file/byte buffer into Paper and retrieve an edited DOCX.

**Acceptance:** fixtures cover paragraphs, styles, lists, tables, images, headers/footers, sections and review content where supported. Tests report known unsupported items and verify no crash on malformed input.

### US-04: Configure the interface
As a developer, I can select a Basic or Advanced preset and customize which toolbar actions are visible.

**Acceptance:** UI visibility is configurable; actual operation permissions are a separate control and are enforced for toolbar, keyboard, context-menu, paste and programmatic command paths where applicable.

### US-05: Fit the editor into my application
As a developer, I can use a full-viewport, embedded or constrained split-pane layout.

**Acceptance:** the editor responds to its container size and does not assume it owns the browser viewport.

### US-06: Own persistence
As a developer, I can save the document output in my own database or object storage.

**Acceptance:** Paper returns documented bytes or text and does not require a Paper account, database schema, storage service or network call.

### US-07: Attribute review activity
As a host application, I can provide an author descriptor for supported comments and tracked changes.

**Acceptance:** display attribution is retained only in formats that support it; the API clearly states that supplied user identity is metadata, not authentication or authorization.

### US-08: Compare versions
As a developer, I can supply two document inputs and request a comparison view or comparison result, subject to engine capability.

**Acceptance:** the audit verifies the current comparison implementation and the SDK exposes it without requiring Paper to store versions. The host application remains responsible for version storage and selection.

## 7. Persistence and file-format requirements

- HTML: interoperability with web applications; potentially lossy for page-level and advanced word-processing structures.
- DOCX: interchange with Microsoft Word and other OOXML tools; fidelity is measured, not assumed.
- PDF: export/distribution where supported by the engine; generally not an editable source format.
- Native Paper format: desired for round-trip editing fidelity, but must be designed and validated before being promised. Do not infer that the existing `paper-docbin` crate is a native persistence serializer; upstream docs describe it as a legacy `.doc` reader.
- JSON: may be useful for inspection, metadata or a structured interchange representation, but do not make it the canonical persistence format until schema completeness and stability are demonstrated.
- Binary APIs should prefer `Blob`, `ArrayBuffer` and `Uint8Array`; Base64 is a transport convenience, not the default storage representation.

## 8. Configuration requirements

The final config API should separate:

- `mode`: a preset such as `basic`, `advanced` or `custom`.
- `toolbar`: the ordered list/groups of visible controls.
- `features`: feature availability at the product level.
- `permissions`: operations allowed for the current editing session.
- `layout`: container sizing, toolbar position, side panels, status bar and page presentation.

Exact property names are provisional and require an API design review. Do not ship a config with two sources of truth for the same setting.

## 9. Non-functional requirements

- No network access required for core editing after assets are loaded.
- Document contents are not sent to Brekquater or third parties.
- Deterministic disposal of editor instances, workers and listeners.
- Clear support matrix for browsers, WASM, clipboard APIs and server-side rendering.
- Typed public errors and documented partial-support warnings.
- Strict TypeScript types for public APIs.
- Tests for corrupt inputs, large documents and unsupported structures.
- Performance baselines for startup, editing latency, layout and memory; do not publish speed claims without reproducible measurements.
- Accessibility limitations of a canvas-based editor must be evaluated explicitly.

## 10. Out of scope

Paper will not initially supply identity management, database persistence, audit log storage, collaborative synchronization, cloud hosting, user billing, license enforcement or legal/regulatory certification.

## 11. Alpha exit criteria

The alpha is ready only when:

1. A fresh developer can follow the docs and run the React example.
2. The demo initializes, loads HTML, edits, exports HTML and disposes cleanly.
3. The demo imports and exports representative DOCX fixtures with documented limitations.
4. A save/load flow works using host-managed storage or an in-memory mock adapter.
5. Rust baseline CI and SDK checks pass in the supported environment.
6. Browser tests cover supported browsers and the editor lifecycle.
7. Public API types and API reference are published in the repository.
8. The product does not claim unverified native binary persistence, exact Word parity, collaboration or unsupported review behavior.
