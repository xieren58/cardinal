# Cardinal

Cardinal 是一套“Rust + Tauri + React”构建的跨平台文件搜索工具链。项目包含一个桌面客户端、命令行/库级索引器以及若干共享组件，目标是在本地磁盘上提供实时、低延迟的搜索体验。

> 如果你刚加入，请先阅读 `doc/AGENTS.md` 获取贡献指南，再回到本文了解整体结构。

---

## 架构速览

```
cardinal/           → 桌面客户端（Tauri + React）
cardinal/src-tauri  → Tauri 后端，调用 Rust 搜索核心
lsf/                → CLI 搜索工具（REPL 模式）
search-cache/       → 核心索引、查询、事件处理逻辑
fswalk/             → 文件树遍历器（Rayon 并行）
cardinal-sdk/       → FSEvent 监听、公共类型
namepool/, query-segmentation/, fs-icon/ … → 辅助库
```

Rust 工作区固定在 `nightly-2025-05-09`，格式化规则见根目录 `rustfmt.toml`。

---

## 功能亮点

- **即时搜索**：首轮全量扫描后，借助 FSEvents 增量同步，保持索引新鲜。
- **虚拟列表渲染**：大规模结果（10k+ 条）也能流畅滚动，详见 `doc/optimizations.md`。
- **按需加载元数据 / 图标**：前端通过 Tauri 命令拉取节点信息与 QuickLook 图标。
- **CLI & GUI 共用内核**：`lsf` 与桌面端共享 `search-cache`。
- **持久化缓存**：退出时写入压缩文件，下次启动快速恢复。

---

## 环境要求

- macOS (当前 FSEvent/图标实现针对 macOS；Windows/Linux 在计划中)
- Rust nightly `nightly-2025-05-09`
- Node.js 18+ & npm（用于 React/Tauri 前端）
- pnpm/yarn 可选（仓库脚本默认使用 npm）

安装依赖：
```bash
rustup toolchain install nightly-2025-05-09
cd cardinal
npm install
```

---

## 快速开始

### 桌面客户端（Tauri）
```bash
# 开发模式（推荐）
cd cardinal
npm run tauri dev -- --release --features dev

# 预览仅限前端
npm run dev

# 打包构建
npm run tauri build
```

### 命令行工具（lsf）
```bash
cargo build -p lsf
cargo run -p lsf -- --path $HOME --refresh
```
- `--refresh` 强制重建索引；不加时优先读取 `target/cache.zstd`。
- 运行时输入 `/bye` 退出。

---

## 测试与验证

- **Rust 单元测试**
  ```bash
  cargo test --workspace
  cargo test -p search-cache  # 关注事件处理、路径去重逻辑
  ```
- **前端性能测试**：详见 `doc/testing.md`，包含 FPS 测量与性能回归步骤。
- **优化记录与验证**：`doc/optimizations.md` 汇总历史优化、关键代码模式与指标。

---

## 重要文档导航

| 文档 | 内容 |
| ----- | ----- |
| `doc/README.md` | 文档索引（快速定位其他主题） |
| `doc/AGENTS.md` | 贡献流程 / 代码风格 / PR 要求 |
| `doc/testing.md` | 性能测试手册与基准场景 |
| `doc/optimizations.md` | 优化记录、细节与快速参考 |
| `doc/todo.md` | 优化与架构层面的待办列表 |
| `doc/cli.md` | CLI 工具（lsf）使用说明 |

---

## 当前重点与待办

- QuickLook 图标缓存与 Viewport 调度优化
- 搜索结果自动刷新、排序与更丰富的查询模式（正则、Glob、普通）
- 元数据后台补水与索引扩展（ctime/mtime/size）
- Windows / NTFS 支持探索

更多细节请查看 `doc/todo.md`。

---

## 支持与反馈

- 提交 Issue 或 PR 前请确保阅读 `doc/AGENTS.md`
- 性能相关问题请附带 Chrome DevTools 录屏或 `cargo test -p search-cache` 结果
- 如需要快速了解近期优化，可先查阅 `doc/optimizations.md`

欢迎贡献和反馈，让 Cardinal 更快、更稳、更好用！
