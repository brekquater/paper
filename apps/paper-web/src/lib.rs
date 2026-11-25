//! Paper in the browser as an embeddable component.
//!
//! A host page owns the `<canvas>` and mounts an editor in it with `PaperEditor.create(canvas)`.
//! Several editors can share a page; each one has its own app, input and frame loop, and
//! `destroy()` (or freeing the JS object) removes its listeners and stops it. The editor shows the
//! full interface with the ribbon by default, or the pages alone with `create(canvas, "pages")`;
//! it opens straight on a blank document, with no start screen (ADR 0003). The host drives it with
//! `execute(command, params)`, opens documents with `load(name, bytes)` and takes them back with
//! `export(format)`. It hears `change`, `selectionchange`, `dirty` and `save` events through
//! `on(event, fn)`: saves inside the editor go to the host instead of downloading. `setAuthor(name)`
//! names the user for comments and tracked changes.
//!
//! Not yet: host configuration of the ribbon, permissions.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::PaperEditor;
