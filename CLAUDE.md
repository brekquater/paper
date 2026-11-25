# Paper — Claude Code Entry Point

@AGENTS.md

## Product documents

Before product or architecture work, read:

- `PRD.md` — scope, user stories and acceptance criteria.
- `ROADMAP.md` — staged delivery and exit gates.
- `docs/PAPER_ARCHITECTURE.md` — target boundaries and integration principles.
- `docs/DATA_FORMATS.md` — import/export and persistence constraints.
- `IMPLEMENTATION_STATUS.md` — current verified status.

`docs/architecture.md` describes the crates as they are; `apps/paper-web/README.md` the browser component.

## Working style

- Default to read-only investigation until the operator authorizes implementation.
- Work in small, testable slices. Do not perform a broad rename or rewrite unless the operator asks; a requested rename is its own checkpoint, never mixed with behaviour changes.
- Do not assume an API exists because it appears in a Paper design example; distinguish proposals from verified upstream capabilities.
- When uncertain, inspect source and tests rather than guessing.
- Keep a concise running note in `IMPLEMENTATION_STATUS.md`; do not treat local scratch plans as authoritative project policy.
- Never publish to npm, create a GitHub release or perform an irreversible action without explicit approval.
- Report outcomes faithfully: say what was verified and what was not ("compiled, not checked in the browser").

## Git and GitHub

- The work ships as **one PR with one commit**, opened only when the project is done. Until then make local checkpoint commits on a `feat/...` branch; push nothing. At the end, squash everything since `master` into one commit and ask before pushing.
- No `Co-Authored-By` or AI attribution lines in commits or PRs. Never write a bare `@name` in commits, PRs or comments.
- Ask before anything outward-facing or hard to reverse: pushing, force pushes, deleting branches, publishing, changing secrets or repository settings. The operator merges PRs; no auto-merge.
- Never print or commit secrets, credentials or personal email addresses.

## Windows environment

- PowerShell is the primary shell; Git Bash is available. Use `python`, not `python3`.
- In Git Bash heredocs backslashes can be lost: write multi-line scripts to a file with the Write tool, then run them.
- Builds and tests need `CRAFT_I18N_DIR` and `CRAFT_FONTS_DIR` (set as user environment variables). Run long builds and test suites in the background.
- Local helper scripts and the next-session to-do live in `plan/` (ignored by git). Dates in docs are absolute (`2026-11-24`).
