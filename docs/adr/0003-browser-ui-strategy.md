# ADR 0003: Paper is an embeddable component over the existing canvas renderer

- Status: Accepted
- Date: 2026-11-22 (proposed); accepted 2026-11-22 by the operator after the baseline audit;
  amended 2026-11-22: the existing ribbon is the default interface, replacing the planned HTML
  toolbar (operator decision after the M1 spike)
- Related: ADR 0002 (preserve the Rust engine), ADR 0004 (persistence), ADR 0005 (framework adapters)

## Context

The baseline audit (2026-11-22) confirmed that the browser build
(the old web app) is the whole egui editor drawn on a `<canvas>` through eframe and wgpu. It
is not a DOM `contenteditable` editor. It can be hosted in two ways today:

- in a same-origin iframe (`?host=parent`, `postMessage`);
- in the host's own page (`data-host="page"`, `CustomEvent`s on the canvas).

Both are driven through a `window` global and the control channel.

The same-page mode can't serve as a component as it is:
- it starts on its own (`main()`), with one hard-coded canvas id (a fixed id);
- page-wide globals: a `thread_local` bridge, a `window` global, and `paste` and `beforeunload`
  listeners leaked with `.forget()`;
- no destroy function;
- exported bytes reach the host only as a save message with no request id.

The iframe mode isolates instances, but it isn't the developer experience Paper is aiming for.

## Options considered

1. Host the existing app in an iframe through its current bridge.
2. Turn the existing canvas editor into an embeddable component, changing Rust where needed.
3. Build a new DOM (`contenteditable`) frontend over the Rust engine.

## Decision

Option 2. Paper is a component that mounts in a host-supplied element, in the style of CKEditor.
It is not hosted in an iframe.

- **No iframe.** A host creates an editor inside its own element (for example
  `create(element, config)`), can run several editors on one page, and tears each one down with
  `destroy()`. The host receives typed events and calls typed commands. The exact API is still
  open (`docs/API_DESIGN.md`).
- **Canvas page area.** The document is drawn and edited by the existing mechanism:
  layout, the `render` rasterizer, and the egui canvas with its caret, selection, typing, mouse,
  pagination and review markup. No HTML or `contenteditable` page area is built.
- **egui dialogs stay in the canvas.** Font, Paragraph, Table Properties and the other existing
  dialogs keep running inside the canvas. They are not rebuilt in HTML. The extra memory this
  costs is accepted for now and will be measured and revisited later.
- **Ribbon by default.** The editor shows the existing interface inside the host's element:
  the ribbon (Home, Insert, Layout, References, Review, View…), rulers, Navigation and other panes,
  and the status bar. Which tabs and commands show is configured through the existing Customize
  Ribbon support (`tools.customizeRibbon`: hide tabs, custom tabs and groups, Quick Access
  Toolbar), exposed as host configuration. The start screen is skipped: the editor opens on a
  document.
- **Pages-only option.** A host that wants its own toolbar can mount the editor with the pages
  alone (`ui: "pages"`, based on Full Screen View) and drive it through engine commands. An HTML
  toolbar in the SDK is optional, later work, not the default.
- **Reuse and prune.** Keep and build on as much of the current code as possible. Features Paper
  doesn't need may be removed or switched off rather than preserved as they are. Each removal is
  its own reviewable change.
- **Rust changes are allowed** where the component needs them. Expected changes, each a separate
  PR with tests:
  1. a library entry point with create and destroy, and per-instance state instead of page-wide
     globals;
  2. the `ribbon` and `pages` interface modes, and ribbon configuration from the host;
  3. byte-level load and export calls;
  4. change, selection and formatting-state events;
  5. author identity and session permissions enforced in `Session::run`;
  6. format errors reported instead of empty output;
  7. host settings that aren't written to `localStorage`.

## Consequences

- The engine, renderer and dialogs are reused rather than rewritten, in line with ADR 0002.
- The page area is a canvas, so screen-reader support, the browser's built-in spellcheck and
  Ctrl+F, and IME input don't come for free. They must be tested and addressed explicitly.
- Each editor instance uses its own GPU context (WebGPU or WebGL2). Browsers limit how many a page
  can hold, so the supported number of instances per page must be measured and documented.
- Keeping egui dialogs in the canvas means a larger wasm download and more memory per instance.
  These are measured as part of the performance baselines.
- Changes to `crates/ui-egui` and the old web app make upstream merges harder. Paper's web
  entry point should live in its own crate, and changes to shared crates should stay small hooks.
- Not yet verified: whether eframe supports several runners on one page and a clean teardown. The
  first spike must prove both before the API is fixed.
