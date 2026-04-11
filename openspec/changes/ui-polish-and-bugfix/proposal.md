## Why

代码 review 发现多项后端 bug（托盘菜单刷新失效、错误处理缺失）和前端 UI 粗糙（无加载态、无暗色模式、空状态简陋、无响应式布局），需要在 v0.2 前集中修复和打磨，提升产品完成度和用户体验。

## What Changes

- 修复托盘 id 不匹配导致菜单刷新失效的 bug
- 修复前端 `cmd_get_config` 无错误处理
- 移除后端死代码（`update_input_name`）
- 添加 MIT LICENSE 文件
- 确保 `package-lock.json` 纳入版本控制
- 提取前后端共享类型定义，消除重复
- 重构前端 UI：加载骨架屏、暗色模式、空状态、Alert 错误提示、响应式网格
- 为输入源重命名添加显式编辑按钮（替代仅双击交互）
- 修复 `index.html` 语言属性为 `zh-CN`

## Capabilities

### New Capabilities

- `ui-polish`: 前端界面美化与交互优化（加载态、暗色模式、空状态、响应式布局、编辑按钮）
- `backend-bugfix`: 后端 bug 修复与代码清理（托盘 id、死代码、错误处理）

### Modified Capabilities

## Impact

- **前端**：`App.tsx`、`monitor-card.tsx`、`index.css`、`index.html`，新增共享类型文件
- **后端**：`tray.rs`、`config.rs`、`lib.rs`
- **根目录**：新增 `LICENSE`，确认 `package-lock.json` 已跟踪
- **无新增依赖**：仅使用已有的 shadcn/ui 组件
