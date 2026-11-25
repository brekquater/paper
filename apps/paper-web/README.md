# paper-web

Paper's editor as an embeddable browser component (ADR 0003): a host page mounts it in a canvas
it owns, can run several on one page, and destroys each one when done.

Status: **M1 spike.** It proves the mechanism; it is not the public API. The TypeScript SDK
(`paper-core`) will wrap it.

## API (wasm-bindgen)

| Call | What it does |
|---|---|
| `await PaperEditor.create(canvas, ui?)` | Mounts an editor with a blank document in `canvas` and resolves when it runs. `ui` is `"ribbon"` (default: the full interface with ribbon, rulers, panes and status bar) or `"pages"` (the pages alone, for a host with its own toolbar). No start screen. |
| `editor.execute(command, paramsJson)` | Runs an engine or UI command by id (`format.bold`, `text.insert`…; `""` for no params) and returns its JSON result. Throws an `Error` on failure. |
| `editor.load(name, bytes)` | Opens a document from a `Uint8Array` **in place of** the current one (its unsaved changes are discarded); `name`'s extension gives the format. Refuses more than 512 MiB. |
| `editor.on(event, fn)` / `editor.off(event, fn)` | Listen for `change` (`{document, revision}`), `selectionchange` (`{document, anchor, focus}`), `dirty` (`{dirty}`) or `save` (`{name, saveAs, bytes}`). Listeners run after the editor's frame, so they may call the editor. |
| `editor.setAuthor(name)` | The name new comments and tracked changes are signed with (up to 200 characters). Display metadata only, not authentication. |
| `editor.export(format)` | Returns the document as a `Uint8Array` in `format` (`docx`, `html`, `pdf`, `odt`, `rtf`, `txt`, `md`). An export is a copy, not a save. |
| `editor.destroy()` / `editor.free()` | Stops the editor: removes its listeners, ends its frame loop and drops it. |

**Saves go to the host.** File › Save, Save As, Ctrl+S and the exports inside the editor never
download: each file arrives as a `save` event, and the host decides where it goes. `saveAs` says
whether the file becomes the document's file. Opening files and pictures from inside the editor
still uses the browser's file picker. Preferences stay in memory.

Not yet: host configuration of the ribbon, permissions, a TypeScript wrapper.

## Build

From the repository root, in PowerShell, with `CRAFT_I18N_DIR` and `CRAFT_FONTS_DIR` set:

1. Build the wasm:
```powershell
cargo build --profile web-release --target wasm32-unknown-unknown -p paper-web
```
2. Generate the JavaScript bindings into `www/pkg/` (wasm-bindgen 0.2.129, the version in `Cargo.lock`; Trunk keeps a copy in its cache):
```powershell
& "$env:LOCALAPPDATA\trunkrs\trunk\cache\wasm-bindgen-0.2.129\wasm-bindgen.exe" --target web --weak-refs --remove-name-section --remove-producers-section --out-dir apps\paper-web\www\pkg target\wasm32-unknown-unknown\web-release\paper_web.wasm
```
3. Serve `www/` and open http://127.0.0.1:8772/:
```powershell
python -m http.server 8772 --bind 127.0.0.1 --directory apps\paper-web\www
```

### Size and serving

The `web-release` profile (workspace `Cargo.toml`) optimizes for size: `opt-level = "z"`, fat LTO, one
codegen unit, symbols stripped; panic messages in the browser lose function names. On 2026-11-25 the
wasm was 69.7 MB raw, 22.2 MB gzip -9 and 14.7 MB Brotli -q 11 (161.7 MB raw and 20.5 MB Brotli with
the plain release profile). **Serve the `.wasm` compressed** (Brotli, precompressed at build time):
the host's web server decides what users download. `python -m http.server` does not compress.

`wasm-opt -Oz` (binaryen 131, with `--enable-bulk-memory --enable-bulk-memory-opt --enable-sign-ext
--enable-nontrapping-float-to-int --enable-mutable-globals --enable-reference-types
--enable-multivalue`) shrinks the raw file to 59.9 MB but the Brotli download grows to 15.2 MB, and
it adds about 8 minutes, so it is not part of the build.

`www/index.html` is the product page: the editor with a sample document, Open and Download
buttons, and a loading bar while the wasm downloads. `.github/workflows/pages.yml` builds it and
publishes `www/` (with the brand kit under `brand/`) to GitHub Pages on every push to `master`.

`www/test.html` is the test page: two editors, each edited through its own ribbon, with host-side
Export DOCX, Load file, Destroy and Mount again buttons, a live view of each editor's events, and
saves from inside an editor downloaded by the page. Add `?test` to show the host buttons and
`?ui=pages` for the pages-only mode.
