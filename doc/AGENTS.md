# Repository Guidelines

## Project Structure & Module Organization
- `cardinal/` hosts the Tauri + React desktop client; UI code sits in `src/`, and native glue lives in `src-tauri/`.
- The Rust workspace (root `Cargo.toml`) tracks `lsf/` (CLI entry), `cardinal-sdk/` (shared types), `fswalk/` (filesystem traversal), `fs-icon/` (icon extraction), `namepool/`, `query-segmentation/`, and `search-cache/`.
- Toolchain pinning (`nightly-2025-05-09`) and import grouping rules sit at the repository root. Keep generated artifacts like `target/` and `cardinal/dist/` out of commits.

## Build, Test, and Development Commands
- `cargo check --workspace` validates all Rust crates; run before pushing cross-crate changes.
- `cargo test --workspace` executes unit and integration suites; narrow scope with `-p <crate>` (e.g., `cargo test -p lsf`).
- `cargo clippy --workspace --all-targets` surfaces lint issues; address warnings or document allowances.
- Front-end flows: `cd cardinal && npm run dev` for the Vite server, `npm run tauri dev -- --release --features dev` for the desktop shell, `npm run tauri build` for release binaries, and `npm run build` for the static bundle.

## Coding Style & Naming Conventions
- Run `cargo fmt --all` to honour repository-wide `rustfmt` settings (grouped crate imports, 4-space indent). Modules, files, and functions stay `snake_case`; types and traits use `PascalCase`.
- Prefer explicit modules over glob imports; rely on `tracing` for structured logs and return `anyhow::Result` from fallible Rust APIs.
- React components in `cardinal/src/components` follow `PascalCase.jsx`; hooks and utilities keep `camelCase` exports inside `kebab-case` folders.
- Use `npm run format` or `npm run format:check` to enforce Prettier defaults (2-space indent, trailing commas) before committing UI changes.

## Testing Guidelines
- Place Rust tests alongside the logic they cover; use a crate-level `tests/` folder for cross-cutting scenarios.
- Run `cargo test --workspace` after touching shared crates, and `cargo test -p lsf` when altering query or indexing code.
- For UI and performance validation, follow `doc/testing.md`: rebuild with `npm run build`, profile in Chrome DevTools, and capture FPS or memory regressions.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (`feat:`, `fix:`, `chore:`). Add scopes when useful (`feat(fs-icon): cache lookups`).
- Squash WIP commits before review. Reference related issues and note impacted crates or UI surfaces.
- PRs should report the `cargo`/`npm` commands executed, attach UI screenshots when applicable, and highlight any risk areas (indexing throughput, icon rendering, search latency).
