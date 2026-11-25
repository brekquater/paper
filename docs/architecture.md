# Architecture

How Paper's Rust workspace is built today. The target design and its boundaries are in
[`PAPER_ARCHITECTURE.md`](PAPER_ARCHITECTURE.md); decisions are in [`adr/`](adr/).

## Layers

`cargo xtask layers` enforces the table below (`xtask/src/layers.rs`): a crate may depend only on
crates in lower layers (and on the earlier crate in an allowed pair within a layer), and only L6
may use UI crates.

| Layer | Crate | Job |
|---|---|---|
| L0 | [`geom`](../crates/geom/src) | Units (points, twips, EMU, inches, cm), rectangles and measurement parsing |
| L1 | [`doc`](../crates/doc/src) | Document model: stories, paragraphs, runs, tables, sections, styles, numbering, fields, notes, comments, revisions; edit primitives |
| L1 | [`fonts`](../crates/fonts/src) | Font database, metrics, outlines, shaping, Word font substitution |
| L2 | [`docbin`](../crates/docbin/src) | Word 97–2003 `.doc` reader |
| L2 | [`docx`](../crates/docx/src) | DOCX reader and writer (Transitional and Strict), equations, custom parts |
| L2 | [`formats`](../crates/formats/src) | TXT, Markdown, HTML, RTF, ODT and LaTeX import/export |
| L2 | [`layout`](../crates/layout/src) | Line layout, page and section flow, tables, hit testing; the display list |
| L3 | [`pdf`](../crates/pdf/src) | PDF export: font subsets, selectable text, links, outline, tagging |
| L3 | [`render`](../crates/render/src) | CPU rasteriser: pages and thumbnails from the display list; PNG/JPEG export |
| L4 | [`engine`](../crates/engine/src) | `Session` and editing state, the command registry (`cmd/`), file I/O dispatch |
| L6 | [`ui-egui`](../crates/ui-egui/src) | The interface: ribbon, keytips, Backstage, dialogs, canvas, rulers, panes |
| app | [`paper-web`](../apps/paper-web/src) | The browser component (`PaperEditor`) |
| tooling | [`xtask`](../xtask/src) | Workspace checks |

External libraries (the `craft-*` crates, egui and eframe) supply the UI toolkit, the shared DOCX
and word-model code, proofing, image codecs and the PDF writer; versions are pinned in the
workspace `Cargo.toml`.

## Data flow

1. **Open:** `engine::io` picks a reader by extension (`docx`, `docbin`, `formats`) and produces a
   `doc::Document`.
2. **Edit:** every user-visible action is a command id run through `Session::run` (ribbon,
   shortcuts, command search and the JavaScript `execute` all dispatch by id). `Session::run`
   snapshots for undo (blocks are `Arc`-shared, so snapshots are cheap) and catches an escaped
   panic, restoring the document.
3. **Layout:** `layout` turns the document into pages of positioned glyphs, boxes and images (the
   display list), incrementally: an edit relayouts only what changed.
4. **Draw:** the interface paints pages through `render` on the egui canvas; PDF export walks the
   same display list through `pdf`.
5. **Save:** `engine::io` picks a writer by extension; in the browser the bytes go to the host as a
   `save` event.

## Testing and gates

- Unit, property and interaction tests per crate (`cargo nextest run --workspace`): model, layout,
  DOCX round-trip and hostile input, `.doc`, formats, typing, bidi, interface catalogs, UI
  interaction (egui_kittest). `hostile_params_never_panic` runs every command with junk parameters.
- `cargo xtask ci`: fmt, clippy `-D warnings`, tests, asset attribution, layering, wasm check.
