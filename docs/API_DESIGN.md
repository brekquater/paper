# Paper Public API — Design Notes

Status: proposal only. Names and types below are illustrative, not implemented APIs.

## Design goals

- Framework-neutral core.
- Typed initialization, lifecycle, document I/O, events and errors.
- No direct exposure of unstable Rust structs.
- No requirement for a Paper backend or storage system.
- Safe SSR behavior: importing the package must not require `window` or `document`.
- Deterministic instance lifecycle and explicit disposal.

## Conceptual API shape

```ts
const editor = await PaperEditor.create({
  container: element,
  mode: 'basic',
  layout: { width: '100%', height: 640 },
});

await editor.load({ format: 'html', data: html });

const html = await editor.export({ format: 'html' });
const docx = await editor.export({ format: 'docx' });

await editor.dispose();
```

Do not implement this exact API until the M1 spike confirms the runtime and control boundaries.

## Lifecycle

The API should distinguish at least: `created`, `loading`, `ready`, `error`, `disposing`, `disposed`. Define behavior for failed initialization and repeated disposal.

## Document I/O

A format-aware API should accept appropriate value types:

- HTML: string.
- DOCX/PDF/binary formats: `Blob`, `ArrayBuffer` or `Uint8Array`.
- JSON: only where the source engine exposes a documented schema with defined stability.

Avoid Base64 as the default binary transport. If Base64 is supported, document it as a convenience conversion.

## Commands and permissions

Separate UI visibility from allowed operations:

- `toolbar.items` controls what controls are presented.
- `permissions` controls which operations are allowed.

Hiding a toolbar button is not a security boundary. The engine/adapter must reject disallowed operations from keyboard shortcuts, context menus, paste paths and programmatic commands where those paths exist.

Do not expose arbitrary raw command execution to untrusted application content. The host application's authorization remains authoritative.

## Events

Provisional event families:

- lifecycle: ready, error, disposed.
- document: changed, loaded, saved/exported.
- selection: selection changed, if supported.
- review: comment/revision changes, if supported.
- warning: unsupported or partially preserved content.

Events must be unsubscribe-able and must not leak document contents in diagnostics by default.

## User/author metadata

A caller may provide an author descriptor for comment/revision attribution if supported by the engine. This is metadata only and does not authenticate the user or authorize edits.

## Version comparison

The host application supplies the two versions. Paper should not own a version database. The API may eventually return a comparison session/view, structured diff summary, or review-marked result. M0 must inspect existing compare/combine functions before choosing the contract.

## Error contract

Define typed errors for unsupported format, malformed input, runtime initialization, permission denied, export failure, unsupported content, disposed instance and timeout/cancellation where relevant. Never convert a failure into a successful empty document without an explicit warning.

## API stability

Before 1.0, document breaking changes and use semver prerelease versions. Keep internal engine command identifiers separate from stable public commands; map them in one adapter layer.
