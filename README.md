# Cardinal

> Local-first file search for macOS, powered by Rust, Tauri, and React.

Cardinal bundles a realtime filesystem indexer, a Tauri desktop client, and a command-line companion (`lsf`). The workspace is written primarily in Rust with a React front-end, aiming to deliver sub-second search across millions of files while staying responsive during incremental updates.

This repository is now being prepared for public release. The documentation below introduces the stack, explains how to run and develop it, and points you to supporting guides listed in this README and under `doc/`.

---

## Table of Contents

1. [Highlights](#highlights)
2. [Repository Layout](#repository-layout)
3. [Getting Started](#getting-started)
4. [Running Cardinal](#running-cardinal)
5. [Testing & Quality Checks](#testing--quality-checks)
6. [Documentation Index](#documentation-index)
7. [Contributing](#contributing)
8. [License](#license)

---

## Highlights

- **Realtime indexing** – A full scan seeds the cache once, after which FSEvents keep results in sync with filesystem changes.
- **Shared core** – Both the desktop app and the `lsf` CLI use the same `search-cache` engine for consistent behaviour.
- **Virtualized UI** – The file list is virtualized and streams metadata/icons on demand to stay smooth with 10k+ results.
- **Extensible workspace** – Supporting crates (`fswalk`, `cardinal-sdk`, `query-segmentation`, `fs-icon`, …) compose the end-to-end pipeline.
- **Persistent caches** – Index state is compressed to disk between runs for quick warm starts.

---

## Repository Layout

```
cardinal/           → Desktop client (Vite + React + Tauri)
cardinal/src-tauri  → Tauri commands bridging to the Rust core
lsf/                → Command-line search tool
search-cache/       → Indexing, querying, and event ingestion logic
fswalk/             → Parallel filesystem traversal utilities
cardinal-sdk/       → Shared types, FSEvent bindings, helpers
fs-icon/, namepool/, query-segmentation/ → Supporting crates
doc/                → In-depth guides, testing notes, and design docs
```

The Rust workspace is pinned to `nightly-2025-05-09`; see `rust-toolchain.toml` and `rustfmt.toml` for formatting rules.

---

## Getting Started

### Prerequisites

- macOS 13 or newer (current icon/FSEvent plumbing targets macOS only)
- [Rust](https://www.rust-lang.org/) toolchain `nightly-2025-05-09`
- [Node.js](https://nodejs.org/) 18+ with npm
- Homebrew dependencies for Tauri (see [Tauri prerequisites](https://tauri.app/start/prerequisites/) if you have not installed them)

### Bootstrap the workspace

```bash
rustup toolchain install nightly-2025-05-09
rustup component add rust-src --toolchain nightly-2025-05-09
cargo install --locked tauri-cli            # optional if you prefer the CLI
npm install --prefix cardinal               # install front-end deps
```

Rust dependencies are vendored via `cargo` when you compile the workspace; no extra steps are needed beyond the toolchain install.

---

## Running Cardinal

### Desktop client (Tauri)

```bash
# Start the desktop shell (Rust backend + webview)
cd cardinal
npm run tauri dev -- --release --features dev

# Optional: run only the Vite front-end
npm run dev

# Produce a signed bundle
npm run tauri build
```

### Command-line search (`lsf`)

```bash
cargo run -p lsf -- --path "$HOME" --refresh
```

`--refresh` forces a new scan; omit it to load the persisted cache in `target/cache.zstd`. Inside the REPL, type `/help` for commands and `/bye` to exit.

---

## Testing & Quality Checks

Run these before sending a pull request or cutting a release:

- **Rust checks**
  ```bash
  cargo fmt --all
  cargo clippy --workspace --all-targets -D warnings
  cargo test --workspace
  ```
  Target a specific crate (e.g. `cargo test -p search-cache`) when iterating on focused changes.

- **Front-end checks**
  ```bash
  cd cardinal
  npm run format
  npm run build
  ```

Performance and regression testing play a big role in keeping the UI smooth. [`doc/testing.md`](./doc/testing.md) describes a profiling workflow (FPS capture, Safari/Chrome tracing) that we follow when changing virtualization, icon loading, or search scheduling.

---

## Documentation Index

- [`doc/README.md`](./doc/README.md) – navigates all available docs.
- [`AGENTS.md`](./AGENTS.md) – in-depth internal guidelines (mirrored in the public [CONTRIBUTING](./CONTRIBUTING.md) notes).
- [`doc/testing.md`](./doc/testing.md) – manual + automated test plans.
- [`doc/optimizations.md`](./doc/optimizations.md) – historical performance work and design decisions.
- [`doc/cli.md`](./doc/cli.md) – `lsf` usage and configuration.
- [`doc/todo.md`](./doc/todo.md) – roadmap and ongoing investigations.

Most documents are currently authored in Chinese; translations can be contributed incrementally. Feel free to open an issue if you need help locating a topic.

---

## Contributing

We welcome issues, bug reports, docs, and code contributions. Please read:

- [CONTRIBUTING.md](./CONTRIBUTING.md) for workflow, coding standards, and commit conventions.
- [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for community expectations.

New contributors can start with documentation fixes, small UI improvements, or targeted tests (see [`doc/todo.md`](./doc/todo.md) for ideas). When filing issues, include platform details and reproduction steps; for performance regressions, attach profiler traces or screenshots when possible.

---

## License

Cardinal is distributed under the [MIT License](./LICENSE). See the license file for details.

---

感谢所有贡献者！如果你更习惯阅读中文，请参考 `AGENTS.md` 以及相关文档；我们也欢迎提交英文/中文双语改进。Happy hacking!
