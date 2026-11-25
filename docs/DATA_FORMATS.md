# Document Formats and Persistence Policy

## Principle

Paper provides editing and conversion capabilities. The host application chooses where and how to store the returned content. Paper must not force PostgreSQL, MongoDB, S3, Firebase, a Paper account or a Paper cloud service.

## Formats and roles

| Format | Role | Important caveat |
|---|---|---|
| HTML | Web interoperability and conventional editor integration | May lose page-level, section-level and advanced Word structures. |
| DOCX | Word-compatible interchange | Round-trip fidelity depends on implemented OOXML coverage. |
| PDF | Distribution/print output | Not an editable source representation. |
| JSON | Inspection or structured interchange if a complete schema exists | Must have schema/version/stability policy before being presented as canonical. |
| Native Paper serialization | Potential best-fidelity save/restore format | Must be discovered or designed; not yet assumed to exist. |
| Base64 | Transport encoding for binary data | Adds size overhead; not the default API/storage form. |

## Critical naming warning

The upstream crate `paper-docbin` is documented as a reader for the legacy Microsoft Word 97–2003 binary `.doc` format. Do not call that crate a native Paper `.docbin` persistence format. Inspect the actual source and I/O registry.

## Native format decision criteria

Before exposing a native save format, establish:

1. Which document fields and embedded assets are serialized.
2. Whether comments, replies, author metadata and tracked changes survive.
3. Whether serialization is deterministic.
4. How schema versions are encoded.
5. Forward/backward compatibility policy.
6. Corruption detection and size limits.
7. Migration strategy across SDK versions.
8. Browser-friendly byte APIs and MIME type.
9. Security treatment for untrusted serialized input.
10. Tests for load-save-load semantic equivalence.

If a stable native format does not already exist, design it behind an explicit ADR. Do not rush to ship a binary contract.

## Expected host application pattern

The host app may persist HTML as text or native bytes in its own database/object store. Paper returns the selected representation; the host app attaches title, owner, timestamps, permissions and any application-specific metadata. Those fields are outside Paper's document payload contract.

## Round-trip test principle

For each supported format, test import → edit → export → re-import. Compare semantic document structures and rendered pages where appropriate. Document known losses explicitly.
