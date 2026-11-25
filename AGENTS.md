# Paper — Agent and Contributor Rules

These rules apply to all human and AI contributors working in the Brekquater Paper fork. The operator's task is authoritative. Use RFC 2119 meanings for MUST, MUST NOT, SHOULD, SHOULD NOT and MAY.

## 1. Product purpose and scope

Paper productizes an existing Rust document engine as an embeddable document editor for web applications.

- The Rust document model, editing/session engine, layout, DOCX/format handling and rendering are existing assets to understand and reuse.
- The initial product work is a framework-independent JavaScript/TypeScript SDK and a React integration.
- Vue and Angular adapters are future work after the shared API stabilizes.
- Paper is initially self-hosted/client-side. Do not introduce a required Paper cloud, hosted database, telemetry service, licensing service or account system.
- Do not start monetization, enterprise licensing, collaboration, real-time multiplayer or cloud hosting work unless the operator explicitly changes scope.

## 2. Deliberate fork-level exception to upstream policy

The inherited engine's agent guidance contained a rule that applications SHOULD be Rust-only and should not contain handwritten JavaScript/TypeScript. Paper's product requirement intentionally introduces a TypeScript SDK.

This is a narrow, explicit exception:

- TypeScript/JavaScript MAY be added under the agreed SDK/package locations after the baseline audit and architecture decision.
- Existing Rust engine crates and desktop/native apps MUST remain Rust-first and MUST NOT be rewritten in TypeScript.
- Do not add Electron, Tauri or a webview shell as part of the initial product.
- Do not weaken Rust safety, layering, licensing, attribution or testing standards to accommodate the SDK.

Where an inherited upstream rule conflicts with the explicit Paper product direction, follow this documented exception only for the specific SDK work. Preserve all non-conflicting upstream safeguards.

## 3. Mandatory first-session procedure

Before changing code, the agent MUST:

1. Read this file and `CLAUDE.md` (if using Claude Code).
2. Read `docs/architecture.md`, `IMPLEMENTATION_STATUS.md`, the ADRs in `docs/adr/` and the browser component's `apps/paper-web/README.md`.
3. Inspect the actual checkout, branch, remotes, working tree and commit SHA. Do not rely on a web summary or stale memory.
4. Identify the exact existing web/WASM entry point, control protocol, build steps and supported browser behavior from source.
5. Run or record the existing baseline checks before modifying source. If a command cannot be run, record why; never claim it passed.
6. Search the repository for existing implementations before adding a parallel one.

The first task is investigation-only unless the operator explicitly authorizes a code spike.

## 4. Non-negotiable engineering principles

- Preserve the existing Rust engine. No rewrite of the document model, editing engine, pagination, layout, DOCX reader/writer or renderer without a written ADR demonstrating a concrete blocker and operator approval.
- Prefer a thin adapter over duplicating existing behavior.
- Keep Rust and TypeScript boundaries explicit and typed. Avoid large, chatty JS↔WASM calls in hot editing paths.
- Use the existing command/session architecture where possible. Do not bypass the engine's established edit, undo or validation paths.
- Maintain upstream layering rules. Do not create dependency cycles or app-to-app crate dependencies.
- Keep changes small and reviewable. One PR should deliver one coherent behavior.
- Do not combine large renames with behavioral changes. Baseline first; rename later, incrementally.
- Do not expose unstable internal Rust structures as a public JavaScript API.
- Do not promise exact Word fidelity. Define supported behaviors and test them with a representative document corpus.
- Never silently discard unsupported document content. Surface a clear warning/error or preserve the original package part where the underlying engine supports it.

## 5. Important document-format distinction

The upstream crate named `paper-docbin` is documented as a reader for legacy binary Microsoft Word `.doc` files. Its name MUST NOT be interpreted as proof that the engine already has a stable native Paper `.docbin` persistence format.

Before defining native persistence, inspect the actual readers/writers in `engine::io`, the document model's serialization support and the web control protocol. A new native binary persistence contract requires an ADR, explicit format versioning, compatibility policy, corruption handling and round-trip tests.

HTML is a useful interoperability format but may be lossy for page-specific, section-level, tracked-change or other advanced document structures. DOCX is the Word interchange format. Do not claim these formats preserve identical information.

## 6. Rust quality and safety

- Follow the repository's existing Rust style, lint configuration and layering checks.
- Do not introduce panics in production code. Avoid `unwrap()`, `expect()`, `panic!()`, `unreachable!()` and `todo!()` in non-test code unless the upstream policy explicitly allows a narrowly justified case.
- Do not silence lints with broad `allow` attributes or weaken workspace lint settings to make a build pass.
- Keep unsafe code out of new code unless an explicit, reviewed need is documented and the repository's existing unsafe-code policy is followed.
- Preserve hostile-input defenses for document files, import data and command parameters.
- Do not change upstream dependency revisions casually. Record dependency changes and rationale.

## 7. TypeScript/JavaScript quality

When SDK code is authorized:

- Use TypeScript with strict compiler settings.
- Keep framework adapters thin; business logic belongs in the framework-independent core.
- Avoid `any`; use explicit types for commands, events, file formats, errors and configuration.
- Design async initialization, disposal, cancellation and error handling explicitly.
- Do not assume browser globals exist during SSR or module import. Make browser initialization explicit and lazy.
- Do not access document content through hidden global state when a typed API can be provided.
- Do not store user documents in local storage by default.
- Do not upload document contents or call external services.
- Keep public API proposals separate from internal implementation types.
- Add runtime validation at untrusted boundaries, including imported data and configuration.
- Document browser support and any limitations of clipboard APIs, fonts, workers and WebAssembly.

## 8. User data, review and identity

- The host application owns authentication, user records, permissions, persistence and version history.
- Paper MAY accept an author/identity descriptor for attribution, but this is display/metadata input, not authentication.
- The host application must authorize operations. Never treat a caller-supplied user ID as proof of identity.
- Preserve comment/revision metadata only where the underlying engine and selected export format support it.
- Do not claim that HTML preserves all review metadata unless tests prove it.
- Do not implement collaboration or CRDT/OT behavior as part of the initial SDK.

## 9. Testing and evidence

- Add regression tests for every behavior changed.
- Rust changes MUST run the narrow relevant test and the repository's prescribed CI gate where feasible. The current upstream README documents `cargo xtask ci`; verify this command in the current fork before relying on it.
- SDK changes MUST include unit tests, TypeScript type checks and browser integration tests for the affected surface.
- Import/export changes MUST include round-trip fixtures and tests for malformed/hostile inputs.
- Clipboard claims MUST be tested in actual supported browsers using real Office-generated clipboard payloads where legally/practically available.
- Compare document structures and rendered output where relevant; a text-only assertion is insufficient for layout fidelity.
- Report exactly what was run and what was not run. Never invent test results.

## 10. Security and privacy

- Treat DOCX, RTF, HTML, images, fonts and other imported content as untrusted.
- Do not execute macros, scripts, external relationships or embedded active content.
- Avoid leaking document content in logs, crash reports, analytics or test snapshots.
- Do not commit secrets, tokens, real customer documents or confidential test fixtures.
- Keep test documents synthetic or appropriately licensed/redacted.
- Ask the operator before publishing packages, creating releases, using secrets, spending money, force-pushing, deleting branches/tags or performing other externally visible/irreversible actions.

## 11. Change workflow

For each task:

1. State the relevant existing behavior and the intended behavior.
2. Identify source files and boundaries before editing.
3. Check for an existing command/API/implementation.
4. Write a small plan for non-trivial work.
5. Implement the smallest coherent change.
6. Add or update tests and docs.
7. Run the relevant checks.
8. Summarize changed files, tests run, limitations and follow-up risks.

Do not create a new npm package just to house a small amount of code. Package boundaries must be justified by independent responsibility, dependency graph or release needs.
