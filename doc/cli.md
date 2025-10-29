# lsf CLI 指南

`lsf` 是 Cardinal 项目的命令行前端，负责本地文件系统的“索引 + 实时增量更新 + 交互式查询”。下列内容覆盖其用途、运行方式与开发注意事项。

## 功能概览
- **全量扫描 + 增量更新**：首次通过 `walk_fs` 构建缓存，随后监听 FSEvents 增量更新，必要时触发 `rescan`。
- **交互式 REPL**：逐行输入查询字符串，返回匹配的路径与元数据。
- **缓存持久化**：退出时写入 `target/cache.zstd`，下次启动优先恢复。
- **日志控制**：支持 `RUST_LOG` 环境变量（默认 INFO）。

## 构建与运行
```bash
# 构建
cargo build -p lsf

# 运行（建议选择较小根目录）
cargo run -p lsf -- --path $HOME --refresh
```
- `--refresh` 强制全量遍历；缺省时尝试读取持久化缓存。
- 运行中输入 `/bye` 退出。
- 启用日志示例：
  ```bash
  RUST_LOG=info cargo run -p lsf -- --path $HOME
  ```

## 命令行参数
- `--path <PATH>`：索引根目录，默认 `/`
- `--refresh`：忽略缓存，重新全量遍历

## 运行流程
1. 启动时尝试加载缓存；失败或 `--refresh` 时重新扫描。
2. 通过 `EventWatcher::spawn` 监听文件系统事件，调用 `cache.handle_fs_events`。
3. 主线程处理用户输入，后台线程负责查询、事件与缓存写入。
4. 退出前写回缓存（`target/cache.zstd.tmp` → `target/cache.zstd`）。

## 注意事项
- 默认根目录为 `/`，首次扫描耗时较长，建议传入家目录/项目目录。
- 当前实现针对 macOS（依赖 FSEvents）；其他平台需要额外适配。
- 重扫触发逻辑、懒元数据补全等后续工作见 `doc/todo.md`。

## 开发提示
- 使用 `RUST_LOG=info,lsf=debug` 观察事件流水。
- 结合 `cargo test -p lsf` 与 `cargo test -p search-cache` 验证修改。
- 可能的改进（摘自源码 TODO）：
  - 查询结果缓存（前缀/后缀/精确）
  - TUI 前端
  - 懒加载元数据、已删除文件回溯等功能。
