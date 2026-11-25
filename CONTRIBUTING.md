# Contributing to Paper

Thank you for helping build Paper. Paper is a productization layer over an existing Rust document engine. Contributions should preserve upstream correctness and avoid duplicate implementations.

## Before opening a PR

- Read `AGENTS.md`, `PRD.md` and the relevant architecture decision records.
- Search the repository for an existing implementation or API.
- Keep the PR focused on one coherent outcome.
- Add tests and update documentation for behavior or public API changes.
- Record upstream files or behavior affected by the change.
- Do not remove upstream notices, attribution, licenses or asset credits.

## Branches and commits

Suggested branch names:

- `audit/<topic>`
- `spike/<topic>`
- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`

Use clear imperative commit messages. Avoid mixing rename-only changes with behavior changes.

## Pull request requirements

Every PR should state:

1. Problem and intended outcome.
2. Why existing functionality could not be reused as-is.
3. Important design choices and affected boundaries.
4. Tests run and tests not run.
5. Browser/OS/toolchain matrix tested.
6. Document-format compatibility or migration impact.
7. Security/privacy implications for imported files or document content.
8. Whether public API behavior changes.

## Publishing and releases

No contributor or AI agent may publish npm packages, create releases or use production credentials without explicit approval from the repository owner.
