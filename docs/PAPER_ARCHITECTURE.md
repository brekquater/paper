# Paper Target Architecture

Status: proposed; validate against the actual fork during M0.

## 1. Guiding boundary

Keep the inherited Rust engine as the document authority. Add a small web integration layer around it. The JavaScript SDK should orchestrate lifecycle and expose stable typed contracts; it should not reimplement layout, document semantics, file parsing or editing behavior already handled by Rust.

## 2. Existing upstream layers to verify

The current upstream architecture describes layers broadly as:

- Geometry/units.
- Document model and editing primitives.
- Fonts and shaping.
- Layout, pagination and format readers/writers.
- Rendering and PDF.
- Engine session, undo and command registry.
- MCP/control interfaces.
- egui UI.
- Desktop, CLI and browser apps.

The exact crate inventory and APIs must be rechecked in the fork. The browser app is currently a WASM path; do not assume it is a DOM `contenteditable` editor.

## 3. Proposed Paper layers

### A. Rust document engine (existing)

Owns document structure, edit semantics, undo/redo, layout, rendering and supported file formats. Keep its internal types private from the public TypeScript contract unless an explicit stable serialization boundary is designed.

### B. WASM/browser integration (existing path to investigate)

Owns the WebAssembly build and browser-host interface. Determine whether to reuse the existing browser application, its host control protocol, a lower-level WASM binding, or a combination. The decision must be evidence-based.

### C. Framework-independent TypeScript core (new)

Potential responsibilities:

- Load runtime and assets lazily.
- Initialize and dispose editor sessions.
- Expose typed lifecycle states.
- Load/import and export documents.
- Execute/query commands through the supported engine interface.
- Map engine events to stable public events.
- Normalize errors and warnings.
- Accept caller-controlled configuration.

It must not depend on React, Vue or Angular.

### D. Editor interface adapter (new or extracted)

The audit must decide whether existing egui/canvas rendering is embedded as-is, exposed through a host surface, or needs a small adapter. Do not replace the renderer merely because a DOM interface seems more familiar. Also do not assume canvas accessibility and selection behavior are adequate; test them.

### E. Framework adapters

React first. Vue and Angular later. Framework packages should only bind component lifecycle, props/events and framework conventions to the same core. No forked editing logic.

## 4. Provisional monorepo

One GitHub repository: `Brekquater/paper`.

Keep the existing Cargo workspace intact. Add JS packages in a separate top-level `packages/` tree only after M0/M1 justify it.

Potential future package names (not final):

- `@brekquater/paper-core`
- `@brekquater/paper-editor`
- `@brekquater/paper-react`
- `@brekquater/paper-vue`
- `@brekquater/paper-angular`
- `@brekquater/paper-documents`

Start with only packages required by the first vertical slice. Prefer three or fewer initial packages; do not create empty packages. Use a single JS workspace and Cargo workspace in the same repository, without forcing the Rust crates into a new structure.

## 5. Runtime lifecycle

The public integration should define and test:

1. Construction/configuration.
2. Asset and WASM loading.
3. Initialization/readiness.
4. Document creation/load/import.
5. Commands and editing events.
6. Save/export.
7. Error and recovery behavior.
8. Disposal and cleanup.

Avoid a public API that requires consumers to manipulate undocumented DOM nodes, global variables or raw Rust command payloads.

## 6. Boundary performance

Measure the number and payload size of JS↔WASM calls during typing, selection, toolbar commands, layout and rendering. Batch non-interactive operations when safe. Keep latency-sensitive editing within the engine's existing command/session path. Do not serialize the entire document after every keystroke unless measurement proves it is acceptable.

## 7. Open architectural questions for M0/M1

- Can the existing browser app be embedded in a host container without owning the whole viewport?
- Is the control protocol stable and safe to expose, or is a lower-level API needed?
- How are assets, fonts, workers and WASM binaries loaded under application CSP?
- Can multiple editor instances coexist in one page?
- What are the SSR/module-import constraints?
- How is document content loaded/saved in the browser path today?
- What is the canonical document serialization, if any?
- Which browser features are needed for clipboard and file operations?
- Does current canvas rendering meet keyboard, screen-reader and focus requirements?

## 8. Data flow (target, subject to audit)

Host app → Paper React adapter → Paper TypeScript core → existing browser/WASM bridge → Rust session/document engine → layout/render/import/export.

The host app owns persistence and metadata. Paper returns a documented content payload and does not decide the database schema.
