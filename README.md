# Paper

Paper is an embeddable document editor for web applications, by Brekquater. A host page mounts the
editor in a `<canvas>` it owns and drives it from JavaScript: open and export Word (DOCX)
documents, run editing commands, listen for changes and receive saves. Everything runs in the
browser; Paper has no server, account or telemetry.

The editor is written in Rust and compiled to WebAssembly: a document model, a layout and
pagination engine, DOCX and other format readers and writers, PDF export, and a ribbon interface
drawn on the canvas.

Status: early development. The browser component works (see [`apps/paper-web`](apps/paper-web));
the typed SDK and the React adapter are next ([`ROADMAP.md`](ROADMAP.md)). Live demo:
https://brekquater.github.io/paper/ (published from `master` by `.github/workflows/pages.yml`).

## Repository

| Path | What it is |
|---|---|
| `crates/` | The Rust crates: model (`doc`), `layout`, formats (`docx`, `docbin`, `formats`), `pdf`, `render`, `engine` (session, commands, file I/O) and the interface (`ui-egui`) |
| `apps/paper-web` | The browser component (`PaperEditor`) and its test page |
| `xtask` | Workspace checks (`cargo xtask layers`, `assets`, `ci`) |
| `docs/` | Architecture, ADRs, data formats, API design and test strategy |
| `paper-brand-assets/` | Paper's logos and icons |

## Build and run

Setup and baseline commands: [`docs/DEVELOPMENT_SETUP.md`](docs/DEVELOPMENT_SETUP.md). Building
the web component and serving it: [`apps/paper-web/README.md`](apps/paper-web/README.md).

## Documents

- [`PRD.md`](PRD.md): scope and requirements.
- [`ROADMAP.md`](ROADMAP.md): milestones.
- [`IMPLEMENTATION_STATUS.md`](IMPLEMENTATION_STATUS.md): what is done and verified.
- [`docs/PAPER_ARCHITECTURE.md`](docs/PAPER_ARCHITECTURE.md) and
  [`docs/architecture.md`](docs/architecture.md): target design and the crates as they are.
- [`CONTRIBUTING.md`](CONTRIBUTING.md), [`AGENTS.md`](AGENTS.md), [`SECURITY.md`](SECURITY.md).

## Licence

Apache License 2.0 ([`LICENSE`](LICENSE)). Paper is derived from an open-source document engine;
its copyright notice and the third-party notices are in [`NOTICE`](NOTICE) and
[`ATTRIBUTION.md`](ATTRIBUTION.md).
