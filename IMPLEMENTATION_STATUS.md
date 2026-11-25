# Implementation Status

Last updated: 2026-11-22
Current milestone: M0 — Fork baseline and technical audit (audit and baseline done; repository provenance still open)
Current branch: `feat/paper-baseline` (from `master` at `4a1c91e`)
Upstream base commit: `33d874c1f5b3e60dfc27bbf9a4c519ab38e8d701` (the upstream repository `main`, 2026-11-22; same tree as `85370b26`). Found by matching `Cargo.toml` / `Cargo.lock`, then the whole tree: the engine source is identical, apart from 9 packaging scripts that lost their executable bit on Windows.
Upstream sync: a local-only `upstream` remote, with pushing disabled. Upstream history is not part of this repository; later upstream changes are taken as a diff from this base commit and applied as one ordinary commit.

> Update this file only with verified facts. Do not mark an item complete because code appears to exist; cite the file/module/test or record the evidence.

## Environment

- OS: Windows 11 Home (10.0.26300), 8 logical cores, 16 GB RAM
- Rust toolchain: rustc / cargo 1.100.0 stable (`x86_64-pc-windows-msvc`), rustfmt 1.11.0, clippy 0.1.100; workspace requires ≥ 1.95
- C/C++: Visual Studio 2022 Build Tools (MSVC) and Windows 10 SDK; needed by `harfbuzz-subset-sys` and `winresource`
- Test runner: cargo-nextest 0.9.152
- Node.js: v26.5.1 (not used yet)
- Package manager: none chosen yet
- Browser(s): none tested yet
- WASM target/tooling: `wasm32-unknown-unknown`; Trunk 0.22.2; wasm-bindgen 0.2.129 (downloaded by Trunk); wasm-opt 131 (Binaryen `version_131`, installed locally after Trunk's download timed out)
- Optional build inputs: craft-i18n at `e56d26244bbac099bfd42e23fb12f78a4be418a7` (`CRAFT_I18N_DIR` points at a checkout); craft-fonts at `b076417b5880bd4154bd22f00695b1d28c6bea83`, the commit upstream CI pins (`CRAFT_FONTS_DIR` points at a checkout)

## Baseline (2026-11-22)

Each `cargo xtask ci` step was run separately, so one failure didn't hide the others. The first
pass ran without the optional inputs; failing steps were rerun with them. The results below are
the final state.

| Check | Command | Result | Notes |
|---|---|---|---|
| Rust formatting | `cargo fmt --all -- --check` | ✅ Pass | |
| Clippy | `cargo clippy --workspace --all-targets --keep-going -- -D warnings` | ❌ Fail: 1 error | Windows only: `xtask/src/stats.rs:113` imports `Counts` and `walk`, used only by a `#[cfg(unix)]` test. Every other crate is clean. Upstream CI runs on Linux. |
| Tests | `cargo nextest run --workspace --lib --bins --tests --no-fail-fast -E "not (package(paper-ui-egui) & test(=i18n::audit::untranslated_interface_text))"` | ✅ Pass with both inputs | 4,718 tests: 4,714 passed, 6 skipped. 4 `ui-egui` tests fail without craft-fonts (missing glyphs `⌘`, `☹`, Arabic `ا`) and pass with it. 1 `engine` test (`packs::tests::swiss_german_and_portugal_portuguese_keep_their_languages_rules`) fails without craft-i18n and passes with it. |
| Interface audit | `cargo xtask i18n-report` | ✅ Pass | |
| Doctests | `cargo test --workspace --doc` | ✅ Pass | 14 crates, **0 doctests** in the workspace |
| Language data | `cargo xtask language` | ✅ Pass | |
| Asset attribution | `cargo xtask assets` | ✅ Pass (after fix) | Failed at first: 47 files in `paper-brand-assets/` had no `ATTRIBUTION.md` row. Fixed 2026-11-22 with Brekquater / original work / Apache-2.0 rows (operator's choice); now all 421 asset files are attributed. |
| Layering | `cargo xtask layers` | ✅ Pass | |
| WASM check | `cargo xtask wasm` | ✅ Pass | `cargo check` only; it doesn't build the old web app |
| Web app build | `trunk build --release` in the old web app | ✅ Pass | Output in `dist/web/`. **The wasm is 109.3 MB (35.0 MB gzipped) after `wasm-opt -Oz`**; `packaging/web/README.md` says about 13 MB (5 MB compressed). The cause is not yet known: the language data is only 3.4 MB, and the release profile has `debug = 0`. |
| Existing browser app | — | Not run | Built, but not yet opened in a browser |

Timings with a cold cache on this machine: first clippy pass 17.3 min; first test build and run
59.4 min; web build 37 min. Changing `CRAFT_I18N_DIR` or `CRAFT_FONTS_DIR` triggers a rebuild of
about 15–30 min.

Cargo 1.100 also prints manifest warnings (unused workspace dependencies `aes`, `arboard`, `hmac`,
`miniz_oxide`, `rayon`, `rfd`, `sha1`, `sha2`; unused `workspace.package` fields `rust-version`,
`repository`). They're warnings only.

## Decisions

- ADR 0003 (Accepted): Paper is an embeddable component in the style of CKEditor, with no iframe.
  The canvas renderer is kept, and the egui dialogs stay in the canvas.
- ADR 0004 (Accepted): the host owns storage; DOCX, through the existing reader and writer, is the
  storage format; HTML is lossy interchange only; no native format for now.

## M1 spike: embeddable component (2026-11-22)

`apps/paper-web` exports `PaperEditor` (`create(canvas)`, `execute`, `load`, `export`,
`destroy`) over the existing `WordApp`, with no ribbon or start screen (Full Screen View).
Tested in the desktop app's browser pane with the test page (now `apps/paper-web/www/test.html`):

| Check | Result |
|---|---|
| Two editors mounted in two host-owned elements on one page | ✅ each mounted in about 770 ms after the wasm loaded |
| Commands stay per editor (`text.insert`, `format.bold`) | ✅ |
| Real keyboard input goes only to the focused editor | ✅ |
| `export('docx')` | ✅ 6,849-byte DOCX; `word/document.xml` holds the text, with the bold run marked `<w:b/>` |
| `load(name, bytes)` of that DOCX into the other editor | ✅ text and bold intact |
| `destroy()`, the other editor keeps working, mount again | ✅ |
| Console errors | none apart from a missing favicon |

Follow-up the same day: the operator made **the existing ribbon the default interface**
(ADR 0003 amended). `create(canvas, ui)` takes `"ribbon"` (default) or `"pages"`. Retested in the
browser pane: both editors show their own title bar, Quick Access Toolbar, tabs (File … Help) and
Home ribbon; the ribbon's Bold button and keyboard typing work per editor; a rebuild after the
change took 7.8 min. The host page's Type text and Bold buttons were removed: editing goes through
the ribbon.

Spike limits, to address in the real implementation:
- ~~`load` opens an extra document instead of replacing the current one.~~ Fixed 2026-11-23 (below).
- In `pages` mode, Full Screen View still shows its search bar and a ruler, and doesn't zoom the
  page to the pane's width.
- ~~The ribbon shows the upstream branding.~~ Removed 2026-11-24/25 (below).
- ~~No events or author.~~ Done 2026-11-23 (below). Still no host configuration or permissions.
- ~~Save inside the canvas downloads.~~ Saves go to the host 2026-11-23. Open and the other pickers
  still use the browser's file dialog.
- The release wasm is 159.1 MB unoptimised. It builds a separate copy of eframe and the craft UI
  crates because its `web-sys` features differ from the old web app's (32.7 min cold build).

### Component API: events, hosted saves, load, author (2026-11-23)

`PaperEditor` gained `on(event, fn)` / `off(event, fn)`, `setAuthor(name)`, and `load` now replaces
the current document. Events run after the frame that raised them (microtask), so a listener may
call the editor. Saves inside the editor (Ctrl+S, File › Save, Save As, exports) go to the host as a
`save` event instead of a download. Tested in the browser pane with the test page, two editors:

| Check | Result |
|---|---|
| Typing in editor A raises `change`, `selectionchange` and `dirty: true` in A only | ✅ (events arrive late while the pane is hidden: the browser throttles frames) |
| `execute("text.insert")` in editor B raises B's own `change` and `dirty` | ✅ |
| Ctrl+S in editor A | ✅ `save` event `{name: "Hello Paper.docx", saveAs: true, bytes}`: a 6,921-byte DOCX whose `word/document.xml` holds the text; `dirty: false`; no download prompt |
| `load(name, bytes)` into editor B | ✅ new document id 7; closing the old id 2 fails with "no open document has id 2" |
| `setAuthor("Editor B user")`, then `review.newComment` | ✅ exported `word/comments.xml` has `w:author="Editor B user"`; the title bar avatar shows the initials |
| Destroy editor A, mount again | ✅ canvas removed, new editor raises its events; editor B keeps working |

### Paper theme, black and white (2026-11-23)

`crates/ui-egui/themes/paper.json` (light and dark) is applied to the craft UI through
`RootEnv::library_theme`, and the app's own colour tokens (`theme/legacy.rs`: title bar, ribbon, tabs,
panels, selection, rulers, balloons) use the same palette; author, find, guide and close-box colours
are unchanged. The interface now **starts in Light** instead of following the system; System and Dark
remain in Options. Checked in the browser pane on a Windows machine in dark mode: both editors open in
Light. Theme, contrast, preference and options tests pass (the one that expected System as the
default was updated).

### Debranding (2026-11-24)

Discord is removed everywhere (title bar, File page, Review ribbon, keytip, option, `ui.discord`
command, icon). About 90 interface strings say Paper instead of the upstream name; the embedded translation
catalogs are renamed the same way at build time (`crates/ui-egui/build.rs`), so translations are kept.
Saved DOCX files name `Paper` as their application. Help › About is Paper's own dialog
(`crates/ui-egui/src/about.rs`): name, version, description, the required upstream licence line
and the language-data licences; the Contributors and Models tabs and
their data are gone. The upstream app icons are removed; the updater and favicons use the Paper
brand kit. The test page shows only the editors; `?test` brings back the host stand-in buttons.
Checked in the browser (no Discord, About dialog); tests of the four changed crates pass (3,055).
Not yet renamed then: crate names, the storage id, the old web app (done 2026-11-25, below).

### Crate rename and old web app retired (2026-11-25)

The 12 engine crates are now `paper-*` / `paper_*` (geom, doc, fonts, layout, docx, docbin, formats,
render, pdf, engine, zotero, ui-egui); the app id and storage id are `paper`; the environment
variables are `PAPER_*` except the dictionaries folder variable, which the external craft-libs
reads under its own name. The old web app and `packaging/web` are removed (paper-web replaces
them). Paper's own fit part in saved DOCX files is now `paperfit`.

### Branding cleanup for the first PR (2026-11-25)

The upstream name appears only in the licence notice (`NOTICE`, the About dialog)
and in names that external code defines: the DOCX passthrough keys and the ODF object marker that
craft-docx and craft-wordmodel write and read, the craft-i18n catalog file name, and craft-proof's
dictionaries variable. Those go when the craft-libs crates are brought into Paper. HTML, RTF and
LaTeX exports name Paper as their generator; ODT exports still name the upstream app in their
metadata and arrow marker names until then. Comments no longer carry upstream issue numbers,
porting notes or contributor handles. Removed: the
upstream logos (trademarks), screenshots and copied docs, the Word-parity and localization notes,
the upstream changelog of gaps, the baseline audit (its findings are in this file and the ADRs) and
starter-kit leftovers. Code comments, test data and test names say Paper; the proofing crate is
used under its own name, `craft-proof`. The sample document links to example.com.

### Product page and GitHub Pages (2026-11-25)

`apps/paper-web/www/index.html` is now the product page (the test harness moved to `test.html`):
Paper's brand colours and mark, light and dark, a loading bar while the wasm downloads, the editor
opening the built-in sample document, Open your DOCX, Download .docx and .pdf, feature cards and an
embed snippet. Checked in the browser at desktop and phone width (no sideways scroll; a note says
the editor is best on a desktop). `.github/workflows/pages.yml` builds the web-release wasm and
publishes `www/` with the brand kit under `brand/`; not run yet, and Pages must be switched to
"GitHub Actions" in the repository settings. The download size GitHub Pages serves (gzip or not)
is to be measured after the first deploy.

Full run after the rename: fmt, workspace clippy (3.3 min), layers (14 crates), assets and xtask
tests pass; **4,585 tests passed, 6 skipped, 0 failed** (46 min, warm dependency cache; the wasm
build took 8.9 min). The rebuilt page was checked in the browser: both editors load, Light theme,
DOCX export 6,821 bytes, Help › About says Paper, no console errors. wasm is 165.5 MB unoptimised.

Found on the way: `cargo clippy --target wasm32-unknown-unknown` fails in upstream
`crates/fonts/src/fontdb/user.rs:659` (`drop(file)` on a type without `Drop` on wasm). Upstream
never runs clippy for wasm.

## Milestones

- [x] M0 — baseline audit (done 2026-11-22)
- [x] M1 — embeddable component proof of concept (ADR 0003; component API done and checked in the browser 2026-11-23)
- [ ] M2 — framework-independent TypeScript core
- [ ] M3 — shared UI and React adapter
- [ ] M4 — document workflows and quality
- [ ] M5 — public alpha

## Verified capabilities

Evidence was recorded in the baseline audit (2026-11-22). "Browser verified" means exercised in a
browser, which hasn't happened yet.

| Capability | Evidence in source/tests | Browser verified? | SDK exposed? | Notes |
|---|---|---:|---:|---|
| WASM/browser startup | the old web app (`src/web.rs`); builds with Trunk | No | No | Starts on its own; one instance per page; no destroy |
| Host control interface | the old web app (`src/api.rs`, `src/host.rs`), `crates/ui-egui/src/control/handler.rs` | No | No | a `window` global plus `postMessage` messages |
| HTML import/export | `crates/formats/src/html.rs` over `craft-html` | No | No | Returns empty output on conversion failure |
| DOCX import/export | `crates/engine/src/io.rs:268`, `crates/docx` over `craft-docx`; 460 docx tests | No | No | Whole document converted through JSON on every load and save |
| PDF export | `crates/pdf`, `file.exportPdf` | No | No | |
| Comments/replies | `crates/engine/src/cmd/review.rs`, `crates/doc/src/review.rs` | No | No | |
| Author attribution | `Session.author`, `file.setAuthor`, `?author=` | Yes (2026-11-23) | `setAuthor` | Display name and initials only; no user id |
| Tracked changes | `crates/engine/src/cmd/review.rs`, `cmd/fmt_revisions.rs` | No | No | |
| Document comparison | `crates/engine/src/cmd/tools/compare.rs:51` | No | No | Revised side only by path or plain text, so no DOCX-vs-DOCX compare in the browser; `review.combine` is the same function |
| Native serialization | `.json` is a serde dump of `Document` (`crates/engine/src/io.rs:252`) | No | No | Not stable; `paper-docbin` is a `.doc` reader only |

## Current blockers

- The repository has no upstream history and nothing beyond `.gitignore` and `LICENSE` is committed.
  Upstream merges aren't possible until the history is rebuilt.

## Known issues (not blockers)

- The web wasm is 109.3 MB (35 MB gzipped), far above the documented 13 MB. Operator decision
  (2026-11-22): not investigated now. Unused desktop and CLI crates were removed (Phase A); all
  editor features are kept, and size will be handled later with build features and on-demand
  assets.

## Next three tasks

1. Rebuild the git history from upstream, then commit the current tree (duplicate fixes, ADRs, audit) in reviewable steps.
2. Operator decision (2026-11-22): **keep every editor feature** (mail merge, citations, macros, speech, ink, SmartArt, WordArt, charts, equations, `.doc` import). Size work comes later through Cargo build features (lean and full builds), not deletion. Fonts and language packs ship **with the package as separate files** served by the host, loaded on demand; no download from the internet. Needs a Rust change so font and language-pack loading reads from a host-configured location, and a check that the update and cloud code makes no calls.
3. M1 spike: two editors mounted in two `<div>`s on one page, an HTML Bold button, `export('docx')` and `destroy()`.

## 2026-11-25: first feature removals

Removed (operator's decision after the feature audit): update checker, font download box, print page and
spooler, Markdown Source view, Read Aloud, and the unused Linux desktop theme reader.

Behaviour kept: File › Export (PDF included) and the engine's Print to PDF; `file.print` (Ctrl+P, mail
merge) opens Export; an unedited Markdown file still saves back unchanged; Move Past Citation;
dictation; the missing-fonts warning. Recorded macros stay for now.

Not removed yet: multi-document window management, crash recovery, the Zotero bridge.

Verified: workspace clippy with `-D warnings`; full nextest run 4,464 tests, 4,463 passed and 6
skipped, with 1 failure (the Arabic mirroring test still looked for the removed Print page; its
section was removed and the test passed on rerun, alone); `xtask layers` (14 crates), `xtask assets`
(403 files) and `cargo test -p xtask` (27); wasm clippy and release build. Browser: both editors
mount with no console errors, typing works, the removed controls are gone, DOCX export gives
6,836 bytes, Ctrl+P opens Export. The wasm went from 165,499,784 to 162,409,662 bytes.

## 2026-11-25: remaining feature removals and one window

Removed: the Zotero bridge (crate `paper-zotero`, the Citations tab; Move Past Citation moved to
References › Citations & Bibliography; Word's own citations stay), crash recovery (File › Recover,
desktop only), AutoSave (the title bar switch and Options row; already off in the browser), and
native multi-window management (View › New Window, Arrange All, Side by Side, Synchronous
Scrolling, and the per-window state swapping). The app draws one window; documents switch in it
(Switch Windows stays, as does Split).

Bug found and fixed on the way: switching documents in one window (the browser's only mode)
dropped the document's unsaved comment drafts, because the canvas reset ran after they were
restored. Caught by `a_refused_comment_draft_stays_with_its_document_on_new`.

Verified: workspace clippy; tests after Zotero/recovery/AutoSave: 4,377 passed; after the window
rework: 4,354 run, 4,353 passed, 1 failed (the bug above), then the 17 comment-draft and
document-switch tests passed with the fix; a full rerun with the fix: 4,354 of 4,354 passed (5 skipped); `xtask layers` (13 crates) and `assets` (395 files);
wasm built (161,658,653 bytes, before the draft fix); browser: editors mount, DOCX export
6,829 bytes, removed commands report unknown, Split and Switch Windows run, no console errors.

## 2026-11-25: browser build size

Added the `web-release` Cargo profile (opt-level "z", fat LTO, 1 codegen unit, strip) and build the
bindings with `--remove-name-section --remove-producers-section`. Measured on the same source:

| Build | raw | gzip -9 | Brotli -q 11 |
|---|---|---|---|
| release (before) | 161.7 MB | 36.0 MB | 20.5 MB |
| release, names stripped | 123.2 MB | 32.5 MB | 19.1 MB |
| release + wasm-opt -Oz | 97.7 MB | 33.3 MB | 20.2 MB |
| web-release | 69.7 MB | 22.2 MB | 14.7 MB |
| web-release + wasm-opt -Oz | 59.9 MB | 23.1 MB | 15.2 MB |

The web-release build took 12.7 min with dependencies already fetched. Checked in the browser: both
editors mount, typing renders, DOCX export gives the same 7,247 bytes. Speed, 150 paragraphs typed
through the command API (hidden browser pane, so noisy): release 196-327 ms, web-release 334-483 ms,
web-release + wasm-opt 245-485 ms. Too noisy to say how much slower size optimization is; measure in a
visible browser on a large document before relying on it. Native builds and tests use the unchanged
`release` and dev profiles.

Speed, measured later the same day with the browser pane visible: one 600-paragraph document
(about 2,400 lines), fresh page per run, 3 runs each, order old/new/new/old/old/new, medians (the
old build's first run was its fastest on open and export):

| Step (command API) | release | web-release (opt-level z) | slower by |
|---|---|---|---|
| open the document | 472 ms (325-559) | 782 ms (751-837) | 66% |
| reformat everything 3 times | 322 ms (289-331) | 457 ms (447-460) | 42% |
| 60 x (type a letter + 2 caret moves) | 973 ms (890-978) | 1,310 ms (1,307-1,326) | 35% |
| export DOCX 5 times | 122 ms (79-136) | 161 ms (160-171) | 32% |

So opt-level "z" costs real speed. Not yet tried: opt-level "s", or "z" everywhere with opt-level 3
for the hot crates (layout, fonts and shaping, document, engine).
