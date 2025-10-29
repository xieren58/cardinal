# 性能优化手册

Cardinal 的性能优化分两批完成：2025-10 的前端渲染优化，以及 2025-12 的缓存与后端算法优化。本文整合原先分散的摘要、细节和对比文档，便于快速回顾历史和继续迭代。

---

## 指标概览

| 指标 | 优化前 | 优化后 | 改善 |
|------|--------|--------|------|
| 滚动帧率 | ~30 FPS | ~58 FPS | **+93%** |
| CPU 使用率（滚动） | ~80% | ~35% | **-56%** |
| 搜索响应 | ~500 ms | ~150 ms | **-70%** |
| icon 更新查找 | O(n) | O(1) | **映射缓存** |
| FSEvent 路径去重 | O(n²) | O(n log n) | **排序 + HashSet** |
| fswalk 文件名提取 | 频繁分配 | 避免额外 PathBuf | **遍历更轻量** |

---

## 2025-12 更新

### icon_update 索引映射缓存
- **位置**: `cardinal/src/hooks/useDataLoader.js`
- **问题**: `Array.indexOf` 在查询 10k+ 结果时退化为 O(n)。
- **方案**: 在 `results` 变化时一次性构建 `Map<SlabIndex, number>`，事件回调 O(1) 定位。
- **效果**: icon viewport 切换时主线程开销显著降低。

核心片段：
```javascript
useEffect(() => {
  const indexMap = new Map();
  resultsRef.current.forEach((value, index) => {
    if (value != null) indexMap.set(value, index);
  });
  indexMapRef.current = indexMap;
}, [results]);
```

### FSEvent 路径去重算法重写
- **位置**: `search-cache/src/cache.rs`
- **问题**: `scan_paths` 依赖 `retain`，深路径链触发 O(n²)。
- **方案**: 为每条路径预计算深度并排序，再用 `HashSet` 判定祖先；新增单测覆盖排序、根路径等场景。
- **效果**: 增量事件批处理稳定，不再卡顿。

核心片段：
```rust
candidates.sort_by(|(path_a, depth_a), (path_b, depth_b)| {
    depth_a.cmp(depth_b).then_with(|| path_a.cmp(path_b))
});
candidates.dedup_by(|(a, _), (b, _)| a == b);
```

### fswalk 文件名提取优化
- **位置**: `fswalk/src/lib.rs`
- **问题**: `entry.path().file_name()` 持续构造 `PathBuf`。
- **方案**: 改为 `entry.file_name()`，仅在递归时构造路径。
- **效果**: 索引构建与 rescan 均减少堆分配。

---

## 2025-10 前端优化

| 优化项 | 说明 |
|--------|------|
| FileRow → `React.memo` | 避免父组件更新导致的整列表重渲染 |
| renderRow → `useCallback` | 稳定函数引用，配合 VirtualList 的 `useMemo` |
| VirtualList 渲染缓存 | 通过 `useMemo` 避免每帧重建项数组 |
| ResizeObserver 防抖 | 100 ms 防抖 + 阈值过滤，减少窗口调整抖动 |
| icon_update 变化检测 | 仅在数据真实变化时创建新 Map 并更新状态 |

详见相关提交历史（2025-10-26）或源文件中的注释。

---

## 快速参考

### React 模式
```javascript
export const FileRow = memo(function FileRow(props) {...});

const renderRow = useCallback((rowIndex, item, rowStyle) => {...}, [deps]);

const renderedItems = useMemo(() => {
  if (end < start) return null;
  return Array.from({ length: end - start + 1 }, ...);
}, [start, end, scrollTop, rowHeight, cache, renderRow]);
```

### 状态更新策略
```javascript
setCache((prev) => {
  const changes = [];
  updates.forEach((update) => {
    const index = indexMapRef.current.get(update.slabIndex);
    if (index === undefined) return;
    const current = prev.get(index);
    if (current?.icon !== update.icon) {
      changes.push({ index, current, newIcon: update.icon });
    }
  });
  if (changes.length === 0) return prev;
  const next = new Map(prev);
  changes.forEach(({ index, current, newIcon }) => {
    next.set(index, current ? { ...current, icon: newIcon } : { icon: newIcon });
  });
  return next;
});
```

### Rust 路径去重要点
```rust
fn has_selected_ancestor(path: &Path, selected: &HashSet<PathBuf>) -> bool {
    let mut ancestor = path.to_path_buf();
    while ancestor.pop() {
        if selected.contains(&ancestor) {
            return true;
        }
    }
    false
}
```

---

## 测试与验证

1. 运行 `cargo test --workspace`（重点 `search-cache`）。
2. 按 `doc/testing.md` 的步骤进行前端性能验证（FPS、窗口调整、图标加载）。
3. 若调整 icon 或路径缓存，补充单元测试覆盖：
   - `scan_paths` 新增场景
   - `useDataLoader` 行为可通过前端集成测试或手动验证

---

## 后续建议

- **路径缓存**：在 `SlabNode` 中缓存绝对路径或前缀，进一步降低 `expand_file_nodes` 的组装成本。
- **QuickLook 图标 LRU**：在 Tauri 后端或 `search-cache` 中维护 `(path, mtime)` → base64 的缓存。
- **重复查询短路**：前端对 `[query, options]` 做去重，避免无效 IPC。
- **元数据补水队列**：在闲时补全 ctime/mtime/size，保持 UI 首屏迅速。
- **Tracing 指标**：结合 `tracing` 记录 `scan_paths` / `expand_file_nodes` 时延，便于长期监控。

如需扩展新优化或回滚，请在提交信息和本文件中同步记录。*** End Patch
