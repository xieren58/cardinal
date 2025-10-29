# 测试与性能指南

## 前端性能测试

### 1. 开发模式测试

```bash
cd cardinal
npm run dev
```

打开浏览器开发者工具 (F12)：
1. 打开 Performance 标签
2. 点击录制按钮
3. 执行以下操作：
   - 快速滚动列表
   - 调整窗口大小
   - 快速输入搜索关键词
4. 停止录制并分析结果

### 2. React DevTools Profiler

安装 React DevTools 浏览器扩展：
1. 打开 Components 标签
2. 切换到 Profiler
3. 点击录制
4. 滚动列表并执行操作
5. 停止录制并查看：
   - 组件渲染次数
   - 渲染耗时
   - 为什么重新渲染

### 3. FPS 监控

在浏览器控制台运行：

```javascript
let lastTime = performance.now();
let frames = 0;

function measureFPS() {
  frames++;
  const currentTime = performance.now();
  if (currentTime >= lastTime + 1000) {
    console.log(`FPS: ${Math.round((frames * 1000) / (currentTime - lastTime))}`);
    frames = 0;
    lastTime = currentTime;
  }
  requestAnimationFrame(measureFPS);
}

measureFPS();
```

## 性能基准

### 预期性能指标

| 场景 | 目标 FPS | CPU 使用率 | 内存增长 |
|------|---------|-----------|---------|
| 静止 | N/A | < 5% | < 1MB/min |
| 滚动 | > 55 FPS | < 40% | < 2MB/min |
| 搜索 | > 50 FPS | < 60% | < 5MB/search |
| 调整窗口 | > 50 FPS | < 50% | Stable |

### 测试场景

#### 场景 1: 大量数据滚动
```
1. 搜索空字符串获取所有文件（10,000+）
2. 快速向下滚动到底部
3. 快速向上滚动到顶部
4. 观察 FPS 是否保持 > 55
```

#### 场景 2: 快速搜索
```
1. 在搜索框快速输入 "test"
2. 观察结果更新的响应时间 (应 < 200ms)
3. 检查是否有延迟或卡顿
```

#### 场景 3: 窗口调整
```
1. 拖动窗口边缘快速调整大小
2. 观察列宽是否平滑更新
3. FPS 应保持 > 50
```

#### 场景 4: 图标加载
```
1. 滚动列表观察图标加载
2. 图标应异步加载不阻塞滚动
3. 检查是否有闪烁
```

## Rust 单元测试

在根目录运行：
```bash
cargo test --workspace
```

重点关注
```bash
cargo test -p search-cache
```
- 覆盖 FSEvent 处理、路径去重（`scan_paths`）、节点展开等逻辑。
- 若新增缓存字段（例如路径缓存、图标缓存），请补充对应单元测试。

`lsf` CLI 可通过
```bash
cargo test -p lsf
```
验证命令行解析与缓存刷新路径。

## 性能回归测试

### 检查点

在每次重大更改后运行：

```bash
# 1. 清理并重新构建
npm run clean
npm run build

# 2. 运行生产构建
npm run tauri build

# 3. 运行应用并执行所有测试场景
```

### 性能对比

优化前 vs 优化后的对比：

| 指标 | 优化前 | 优化后 | 改善 |
|-----|-------|-------|-----|
| 滚动 FPS | ~30 | ~58 | +93% |
| 搜索响应 | ~500ms | ~150ms | -70% |
| CPU (滚动) | ~80% | ~35% | -56% |
| 内存稳定性 | 差 | 优 | ++ |

## 常见性能问题排查

### 问题 1: 滚动卡顿

**可能原因**:
- FileRow 组件没有 memo
- renderRow 函数引用不稳定
- 图标加载阻塞渲染

**排查步骤**:
1. 检查 React DevTools Profiler 中 FileRow 的渲染频率
2. 确认 renderRow 使用了 useCallback
3. 验证图标加载是否异步

### 问题 2: 内存泄漏

**可能原因**:
- ResizeObserver 未正确清理
- 事件监听器未移除
- Map/Set 持续增长

**排查步骤**:
1. 使用 Chrome Memory Profiler
2. 记录堆快照前后对比
3. 检查 useEffect 的清理函数

### 问题 3: 搜索响应慢

**可能原因**:
- 搜索请求未防抖
- 结果更新触发大量重渲染
- Tauri 端处理慢

**排查步骤**:
1. 添加搜索时间日志
2. 检查网络标签中的 IPC 调用时间
3. 验证结果缓存是否生效

## 性能优化检查清单

- [ ] FileRow 使用 React.memo
- [ ] renderRow 使用 useCallback
- [ ] VirtualList 渲染项使用 useMemo
- [ ] ResizeObserver 使用防抖
- [ ] Icon 更新有变化检测
- [ ] 没有不必要的状态更新
- [ ] 所有 useEffect 有正确的清理
- [ ] 大型计算结果被缓存

## 工具推荐

1. **Chrome DevTools Performance**: 录制运行时性能
2. **React DevTools Profiler**: 分析组件渲染
3. **Chrome Memory Profiler**: 检测内存泄漏
4. **why-did-you-render**: 调试不必要的重渲染
