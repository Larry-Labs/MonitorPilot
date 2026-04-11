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
- 主题色从纯灰色替换为蓝紫色品牌色方案
- 功能提示面板改为仅首次启动显示，关闭后持久化
- 将 m1ddc 二进制打包进 app（macOS 用户零配置）
- 修复 m1ddc 错误信息捕获（stdout vs stderr）
- 过滤内置显示器（不支持 DDC/CI 切换）
- 完善托盘菜单：添加版本标题、自定义名称、关于菜单、emoji 图标

## Capabilities

### New Capabilities

- `ui-polish`: 前端界面美化与交互优化（加载态、暗色模式、空状态、响应式布局、编辑按钮、品牌色、首次提示）
- `backend-bugfix`: 后端 bug 修复与代码清理（托盘 id、死代码、错误处理、m1ddc 打包）
- `tray-polish`: 托盘菜单完善（版本标题、自定义名称、关于菜单）

### Modified Capabilities

## Impact

- **前端**：`App.tsx`、`monitor-card.tsx`、`index.css`、`index.html`，新增共享类型文件、骨架屏组件、功能提示组件
- **后端**：`tray.rs`、`config.rs`、`lib.rs`、`monitor.rs`
- **打包**：新增 `src-tauri/binaries/m1ddc-aarch64-apple-darwin`，`tauri.conf.json` 增加 `externalBin`
- **根目录**：新增 `LICENSE`
- **新增依赖**：`tauri-plugin-shell`（用于 sidecar 支持）、`@tauri-apps/plugin-shell`
