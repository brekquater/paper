# Provenance, attribution and licence handling

Paper is licensed under the Apache License 2.0 (`LICENSE`). It is derived from an open-source
document engine offered under MIT OR Apache-2.0, which Paper uses under Apache-2.0. The engine's
copyright notice is kept in `NOTICE` and the About dialog; those are the only places that name it.

## Rules

- Keep `LICENSE`, `NOTICE` and `ATTRIBUTION.md` with every copy of the
  source, every binary and every npm package.
- Keep copyright notices in source files and copied assets.
- Use Paper/Brekquater branding only. The upstream project's name, logos and marks are not covered
  by the code licence; do not use them or imply endorsement or affiliation.
- External libraries (the `craft-*` crates, the patched egui, eframe and winit) are pinned Git
  dependencies under permissive licences; review their licences when a pin changes.
- Some names defined by those libraries stay as they are until Paper owns the code: the DOCX
  passthrough keys they write and read, and the interface catalog file name in craft-i18n.

## Before the first public release

- [ ] Confirm licence obligations for all Rust crates, assets and third-party dependencies.
- [ ] Confirm how `NOTICE` and the licence files are carried into npm packages and binaries.
- [ ] Verify that generated bundles include the required notices.
