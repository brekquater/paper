# ADR 0004: Host-owned persistence; DOCX is the storage format

- Status: Accepted
- Date: 2026-11-22 (proposed); accepted 2026-11-22 by the operator after the baseline audit
- Related: `docs/DATA_FORMATS.md`, ADR 0003

## Context

The product must be easy to embed into existing applications. Developers decide how and where
content is stored. HTML and DOCX are important interchange formats, but they don't preserve the
same document semantics.

The audit (sections 6–8) found:

- **DOCX** is read and written through the existing `paper-docx` crate, which delegates to
  `craft-docx` (`crates/engine/src/io.rs:268`, `crates/docx/src/lib.rs`). It keeps comments,
  replies, tracked changes, sections, headers and footers, unknown package parts (`passthrough`),
  embedded fonts and encryption, and has 460 tests including round-trip and malformed-input cases.
- **HTML** import and export work, but can lose page-level, section-level and review structure.
  Today they also return an empty document or an empty string when the internal conversion fails
  (`crates/formats/src/html.rs:14-40`).
- **No stable native format exists.** The `.json` / `.wcraft.json` format is a direct serde dump
  of the internal `Document` (`crates/engine/src/io.rs:252`, `:461`): no version field, lenient
  defaults, images inline as base64.
- **`paper-docbin`** is a legacy Word 97–2003 `.doc` reader only. It writes nothing.

## Decision

1. **Paper does not own storage.** It has no database schema, storage backend or account. The SDK
   hands documents to the host as bytes (`Uint8Array`, `ArrayBuffer` or `Blob`) and the host
   decides where they go.
2. **DOCX is the storage format.** Paper uses the existing DOCX reader and writer, unchanged in
   format, as the format a host saves and reloads. Loading what Paper saved must give back the same
   document, comments and tracked changes included.
3. **HTML is for interchange, not storage.** `getData('html')` / `setData(html)`-style calls are
   offered for web integration, documented as lossy. Format failures must be reported as errors,
   never as empty output.
4. **No native Paper format for now.** The internal `.json` dump is not exposed as a public or
   storage format. A native format needs its own ADR with versioning, compatibility, corruption
   handling and round-trip tests.
5. **Metadata stays with the host.** Title, owner, permissions, timestamps and version history are
   the host's. Paper keeps only what DOCX itself carries (for example comment and revision authors
   and dates).

## Consequences

- Paper fits any host storage, and saved files open in Microsoft Word and other OOXML tools.
- DOCX fidelity is the product's fidelity. It must be measured with a representative fixture
  corpus (load → edit → save → reload) and the known losses documented; exact Word parity is not
  promised.
- Every DOCX load and save currently converts the whole document through JSON
  (`crates/docx/src/model_bridge.rs:10`). Time and memory for large documents must be measured.
- Whether paragraph-level kept markup survives a DOCX round trip is unverified (audit section 8)
  and must be tested before Paper claims unknown content is preserved.
- Host-supplied user ids aren't stored today: DOCX comments and revisions carry a display name and
  initials, not an id. Mapping host users to authors is a separate API decision.
- Base64 is not the default binary representation.
- The HTML silent-empty-output behaviour has to be fixed in Rust before HTML is offered in the SDK.
