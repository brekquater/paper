# Test Strategy

## 1. Test layers

### Existing Rust engine

- Preserve the upstream unit, integration, property and hostile-input tests.
- Run the repository's own CI command; current upstream docs identify `cargo xtask ci`, but verify this in the checked-out fork.
- Add regression tests to the lowest layer that owns the behavior.

### WASM/browser integration

- Runtime initialization and failure paths.
- Asset loading and CSP behavior.
- Multiple instances and repeated mount/unmount.
- Command dispatch and event mapping.
- Document load/save/export.
- Browser resize, focus, keyboard and clipboard behavior.
- Worker cleanup and memory lifecycle.

### TypeScript SDK

- Strict TypeScript compile.
- Unit tests for configuration validation, lifecycle, errors, format adapters and event subscriptions.
- SSR import test: importing package must not touch browser globals.
- No framework dependency in the core package.

### React adapter

- Mount/unmount and Strict Mode behavior.
- Re-render with changed props/configuration.
- Stable callbacks and subscriptions.
- Cleanup on unmount.
- Error boundary / initialization failure behavior.

### Document fidelity

- HTML fixtures.
- DOCX fixtures with paragraphs, styles, list numbering, tables, images, sections, headers/footers, footnotes/endnotes and comments/revisions where supported.
- Malformed and hostile file fixtures.
- Files with unsupported structures to verify warning/preservation behavior.
- Compare structural and visual output, not just extracted text.

## 2. Test corpus governance

Use synthetic, public-domain, permissively licensed or redacted fixtures. Do not add confidential customer documents. Record fixture origin and license. Keep a small smoke corpus and a larger optional regression corpus.

## 3. Clipboard matrix

Test real clipboard payloads originating from Microsoft Word and supported browsers/OS combinations. Record source application/version, browser/version and OS. Include plain text, formatted paragraphs, lists, tables, images and hyperlinks where feasible. Browser clipboard access may differ; do not assume the same behavior across browsers.

## 4. Performance

Record baseline numbers before optimization:

- Cold and warm initialization.
- WASM download/compile/instantiate time.
- Time to open representative documents.
- Typing latency and layout latency.
- Time and memory for DOCX import/export.
- Long-document scroll/render behavior.
- JS↔WASM call count and payload size.

Benchmarks must be reproducible and identify hardware/browser/build mode. Avoid unsupported marketing claims.

## 5. Required report for each PR

List exact commands executed, pass/fail status, skipped tests and reasons. Include fixture names and browser matrix. Never state that tests passed unless they were run.
