# Cardinal

The open-source and cross-platform Everything.

The event processor is sync and it uses dedicated threads.

Only paths are scanned and stored since fetching metadata is slow.

```
cbindgen --config cbindgen.toml --crate cardinal --output cardinal.h

cargo run --release --example tree
```

TODO: 

[] merge should check access(create) time and event time.(however it's difficult on macOS since we have no way to get timestamp of a fsevent, maybe double check if it's present or not on merging?).
[] only use fsevent for diff trigger rather than diff data provider? (which means ignore it's delete, modify or create, just checking the place.).
[] use jwalk to maximize walkdir's performance?
[] use tauri for UI layer

Native GUI，cross platform core.
Compressed BTree file system data on saved.

// touch tmp.md(uncreated):
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/tmp.md",
        flag: kFSEventStreamEventFlagItemCreated | kFSEventStreamEventFlagItemIsFile,
        id: 281055642,
    },
]

// touch README.md(created)
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/README.md",
        flag: kFSEventStreamEventFlagItemInodeMetaMod | kFSEventStreamEventFlagItemIsFile,
        id: 313783990,
    },
]

// rm tmp.md
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/tmp.md",
        flag: kFSEventStreamEventFlagItemRemoved | kFSEventStreamEventFlagItemIsFile,
        id: 281056153,
    },
]

// mv tmp.md emm.md
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/tmp.md",
        flag: kFSEventStreamEventFlagItemRenamed | kFSEventStreamEventFlagItemIsFile,
        id: 281060640,
    },
    FsEvent {
        path: "/Users/bytedance/code/cardinal/emm.md",
        flag: kFSEventStreamEventFlagItemRenamed | kFSEventStreamEventFlagItemIsFile,
        id: 281060641,
    },
]

// chmod +x emm.md
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/emm.md",
        flag: kFSEventStreamEventFlagItemChangeOwner | kFSEventStreamEventFlagItemIsFile,
        id: 281063105,
    },
]

// Change README.md and save
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/README.md",
        flag: kFSEventStreamEventFlagItemInodeMetaMod | kFSEventStreamEventFlagItemModified | kFSEventStreamEventFlagItemIsFile,
        id: 286980245,
    },
]

// ln -s README.md readme
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/readme",
        flag: kFSEventStreamEventFlagItemCreated | kFSEventStreamEventFlagItemIsSymlink,
        id: 286990858,
    },
]

// rm readme (symlink)
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/readme",
        flag: kFSEventStreamEventFlagItemCreated | kFSEventStreamEventFlagItemRemoved | kFSEventStreamEventFlagItemIsSymlink,
        id: 286993883,
    },
]

// ln README.md readme
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/readme",
        flag: kFSEventStreamEventFlagItemCreated | kFSEventStreamEventFlagItemIsFile | kFSEventStreamEventFlagItemIsHardlink,
        id: 286993132,
    },
    FsEvent {
        path: "/Users/bytedance/code/cardinal",
        flag: kFSEventStreamEventFlagItemInodeMetaMod | kFSEventStreamEventFlagItemIsDir,
        id: 286993135,
    },
]

// rm readme (hardlink)
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/readme",
        flag: kFSEventStreamEventFlagItemRemoved | kFSEventStreamEventFlagItemIsFile | kFSEventStreamEventFlagItemIsHardlink,
        id: 286993483,
    },
]




// touch target
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target",
        flag: kFSEventStreamEventFlagItemInodeMetaMod | kFSEventStreamEventFlagItemIsDir,
        id: 316120318,
    },
]

// mkdir target/tmp
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target/tmp",
        flag: kFSEventStreamEventFlagItemCreated | kFSEventStreamEventFlagItemIsDir,
        id: 281055860,
    },
]

// rm -r target/tmp
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target/tmp",
        flag: kFSEventStreamEventFlagItemRemoved | kFSEventStreamEventFlagItemIsDir,
        id: 281060345,
    },
]

// mv target/tmp target/emm
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target/tmp",
        flag: kFSEventStreamEventFlagItemRenamed | kFSEventStreamEventFlagItemIsDir,
        id: 281061482,
    },
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target/emm",
        flag: kFSEventStreamEventFlagItemRenamed | kFSEventStreamEventFlagItemIsDir,
        id: 281061483,
    },
]

// chmod 600 target/emm
[
    FsEvent {
        path: "/Users/bytedance/code/cardinal/target/emm",
        flag: kFSEventStreamEventFlagItemChangeOwner | kFSEventStreamEventFlagItemIsDir,
        id: 286155411,
    },
]

# Cardinal

High‑performance, cross‑platform local filesystem index & instant search ("an open-source Everything").

## ✨ Overview
Cardinal maintains a local index of your filesystem (paths first, metadata lazily) and serves ultra‑fast substring/glob/regex style searches (extensible) through a Tauri desktop UI. It aims for:
* Fast initial scan & incremental live updates (FSEvents on macOS; extensible to other platforms)
* Low memory representation (compressed / structured trees)
* Smooth rendering of very large result sets via a custom virtual list
* Minimal latency interactive search with intelligent loading UI

## 🧱 Architecture
| Layer | Tech | Responsibility |
|-------|------|---------------|
| Core Index / Watch | Rust (multiple crates) | Scan FS, watch events, maintain in‑memory index, provide query APIs |
| IPC Boundary | Tauri (Rust <-> JS) | `invoke` commands: `search`, `get_nodes_info`, `open_in_finder` |
| UI | React + Vite | Search box, virtualized results, column resizing, context menus, status bar |

### Data Flow
1. User types → `useSearch` (debounced) → `invoke('search')` returns a light list (paths / ids).
2. Virtual list asks `useRowData.getItem(index)` → triggers batched `invoke('get_nodes_info')` for uncached rows.
3. Status updates stream via `listen('status_bar_update')` (files scanned, events processed).
4. Column layout driven by CSS variables synced from `useColumnResize`.

## 🖥️ Frontend Structure
```
cardinal/src
    App.jsx               # Composition root
    hooks/                # State & data hooks (search, row data, resize, context menu)
    components/           # VirtualList, FileRow, ColumnHeader, StatusBar, etc.
    constants/            # Layout + tuning constants (row height, overscan ...)
    utils/format.js       # Byte formatting
```

### Key Hooks
* `useSearch` – debounced querying, staged loading indicator (immediate for first search, delayed for subsequent to avoid flicker), duration metrics, error handling.
* `useRowData` – sparse row detail cache; lazy batch fetch of metadata (mtime, ctime, size) only when needed.
* `useColumnResize` – (optimized) rAF‑throttled drag resizing + auto fit + adaptive fallback on extreme window shrink.
* `useContextMenu` – unified context menu state for file rows and header (reset columns, open in Finder).

### Virtual List (`VirtualList.jsx`)
* Computes visible window + overscan, absolutely positions rows.
* Custom vertical scrollbar (track + thumb) with drag + page click.
* Horizontal scroll remains native (overflow-x) enabling smooth column width usage.
* Exposes imperative API: `scrollToTop`, `scrollToIndex`.

### Rendering & Highlighting
* `MiddleEllipsisHighlight` splits matched substrings and applies middle truncation while preserving highlight boundaries.
* Approximate character width heuristic (`CHAR_WIDTH`) – can be upgraded to measurement for perfect fit.

## ⚙️ Performance Techniques
| Concern | Strategy |
|---------|----------|
| Large result sets | Virtualized rendering (only visible + overscan rows) |
| Scroll perf | rAF‑coalesced resizing (new), minimal state for scroll top & range |
| Metadata cost | Two‑phase: initial paths → lazy fetch metadata slices |
| Search latency UX | Delayed loading indicator (150ms) after first search to suppress flicker |
| Column layout | CSS vars update instead of recalculating grid templates in multiple places |
| Drag churn | rAF batching of width updates (avoid per‑mousemove re-render) |

## 🔧 Development
Prerequisites: Rust toolchain, Node.js, (recommended) pnpm or npm, Tauri environment deps (see Tauri docs per OS).

Install & Run (release backend for closer real-world perf):
```bash
npm install
npm run tauri dev -- --release
```

Pure web (no Tauri backend – limited functionality):
```bash
npm run dev
```

Generate C header (if embedding):
```bash
cbindgen --config cbindgen.toml --crate cardinal --output cardinal.h
```

Tree example (debugging index shape):
```bash
cargo run --release --example tree
```

## ✅ Recent Improvements
* Column resizing rAF batching + adaptive recompute on narrow windows.
* Clear separation of lazy metadata fetching vs. initial path array.
* Middle ellipsis with highlight aware splitting.

## 🗺️ Roadmap / Ideas
Core / Index
* [ ] Merge logic: reconcile access/create timestamps vs. event times (macOS FSEvents lacks per‑event timestamp granularity → may need presence checks).
* [ ] Option: treat FSEvents solely as invalidation triggers (re-scan localized path) vs. trusting type flags.
* [ ] Evaluate `jwalk` / parallel walk to accelerate initial scan.
* [ ] Metadata indexing expansion (ctime / mtime / size already partly integrated; consider extended attributes, file type icons cache).
* [ ] Replace bincode with postcard for smaller persistent footprint; explore `IndexSet` / slab structures for name & metadata caches.
* [ ] Windows (USN Journal) & Linux (inotify / fanotify) backends.

Search / Query
* [ ] Unified mode detection: substring / glob / regex → auto infer + manual toggles.
* [ ] Live updating search results as new events arrive (incremental apply instead of full refresh).
* [ ] Rust-side pre‑highlight (return match spans to reduce JS work).
* [ ] Abortable in‑flight search with `AbortController` to avoid stale late responses.

UI / UX
* [ ] Persist column widths (`localStorage` / Tauri store).
* [ ] Native file icons per row.
* [ ] Keyboard navigation & accessibility roles (`grid`, `row`, `gridcell`).
* [ ] Optional exact width measurement for ellipsis (canvas measure / binary search). 
* [ ] Batch metadata prefetch on range change (instead of per‑miss heuristics) to reduce redundant invokes.
* [ ] Consider reverting to native vertical scroll w/ custom styling (simplify wheel handling, better touchpad inertia) OR throttle wheel handler with rAF.
* [ ] Enhanced error panel (copy details, retry button).

Performance / Tuning
* [ ] Evaluate overscan adaptively (dynamic based on scroll velocity).
* [ ] Warm cache for top N most frequent directories.

## 🔍 Design Choices
* Store paths early – defer expensive metadata I/O until user actually views rows.
* Favor simple data shapes over aggressive premature compression (optimize once profiling identifies hotspots).
* UI relies on CSS custom properties for single-pass layout recomputation when widths change.

## 🤝 Contributing
1. Fork & create feature branch.
2. Keep changes small; add notes if touching core indexing algorithms.
3. Run dev mode & ensure no console errors.
4. Open PR with before/after perf notes when relevant.

## 📦 License
TODO: Add a license (MIT / Apache-2.0 recommended for Rust + Tauri ecosystems).

## 🧪 FSEvents Examples (macOS)
Collected raw event samples for reference:
<details><summary>Expand FSEvents Samples</summary>
