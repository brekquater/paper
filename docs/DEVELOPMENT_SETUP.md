# Development Setup and Baseline Commands

This document intentionally avoids inventing installation commands. The inherited project has platform-specific dependencies and its own build orchestration. The checked-out fork's README, Cargo manifests and CI workflow are authoritative.

## 1. Inspect toolchain requirements

From the repository root:

```bash
git status --short
git rev-parse HEAD
git remote -v
rustc --version
cargo --version
rustup show
```

Inspect:

- `Cargo.toml` and any `rust-toolchain.toml`.
- `.cargo/config.toml` and `xtask` documentation.
- `README.md` build prerequisites.
- `.github/workflows/` CI commands.
- `apps/paper-web` build instructions and browser assets.

The current upstream Cargo manifest has declared edition 2024 and a Rust minimum; verify the exact version in your local fork rather than pinning a remembered value.

## 2. Establish the untouched baseline

Use the exact commands documented by the checked-out repository. Current upstream README/architecture documentation identifies `cargo xtask ci` as the full CI gate, covering formatting, clippy, tests, attribution/layer checks and a WASM build. Verify its behavior and prerequisites in the fork before running it.

Record:

- Command and working directory.
- OS and tool versions.
- Exit code.
- Duration if useful.
- Any skipped steps and reason.
- Logs/artifacts, with private content removed.

Do not alter project configuration to make the first baseline pass. Record the failure first.

## 3. Node/TypeScript toolchain

Do not add a JS package manager, Node version file or root package manifest until M0/M1 has selected the SDK boundary and package manager. At that point:

- Choose one supported Node LTS line.
- Choose one package manager and commit its lockfile.
- Use strict TypeScript settings.
- Keep the Rust and JS build/test jobs explicit.
- Pin reproducible tool versions in CI.

## 4. Browser verification

Document the exact command to build and run the existing browser app. Verify its WASM assets, fonts, workers and static resources load under the local development server. Record browser version and any required headers/CSP exceptions.

## 5. Do not do during baseline

- Do not rename crates or Rust symbols.
- Do not add npm packages.
- Do not reformat the whole repository.
- Do not update every dependency.
- Do not delete inherited CI or packaging files.
- Do not publish artifacts.
