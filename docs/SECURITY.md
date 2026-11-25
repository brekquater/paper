# Security and Privacy Requirements

Paper handles potentially sensitive business and personal documents. Security must be considered at every file and browser boundary.

## Data handling

- The core SDK must not upload document content or metadata to Brekquater or a third party.
- No required telemetry, analytics, account, license or cloud call in the initial product.
- Do not persist document contents in localStorage/sessionStorage by default.
- Do not log document text, comments, user content or full imported file bytes.
- Keep diagnostics useful without containing document content.

## Untrusted documents and content

Treat DOCX/OOXML, RTF, HTML, images, fonts, archives, embedded objects, relationships and native serialized data as untrusted.

- Do not execute macros or scripts.
- Do not automatically fetch external relationships/resources without an explicit, reviewed policy.
- Sanitize/validate HTML at the correct boundary and document which input is trusted.
- Enforce reasonable size, nesting, decompression and resource limits.
- Handle malformed files without panic, crash or uncontrolled memory allocation.
- Avoid unsafe deserialization assumptions.

## Browser security

- Document CSP, worker and WASM requirements.
- Avoid `eval`, dynamic code execution and unnecessary remote scripts.
- Do not place secrets in browser bundles.
- Do not claim client-side permission checks protect against a malicious end user; the host server must enforce authorization for stored documents and actions.

## Identity and review

Author IDs/names supplied to Paper are attribution metadata only. They do not authenticate a user. The host application must authorize edits, comments, exports and access to versions.

## Dependency and release security

- Keep dependencies minimal and reviewed.
- Preserve upstream security fixes.
- Do not publish artifacts until reproducible builds and package contents have been reviewed.
- Ask the operator before handling production credentials or publishing.
