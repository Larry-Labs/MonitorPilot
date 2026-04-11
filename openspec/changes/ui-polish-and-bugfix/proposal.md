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
- 主题色从纯灰色替换为蓝紫色品牌色方案（oklch 色彩空间）
- 移除功能提示面板（用户反馈不需要）
- 将 m1ddc 二进制通过 Tauri externalBin 打包进 app（macOS 用户零配置）
- 修复 m1ddc 错误信息捕获（stdout vs stderr）
- 过滤内置 MacBook 显示器（不支持 DDC/CI 切换）
- 完善托盘菜单：版本标题、自定义名称、帮助子菜单（关于/主页/反馈）、移除 emoji
- 实现 Toast 通知系统：底部提示条，区分切换中/成功/警告/失败四种状态
- 实现热插拔检测：5s 定时轮询 + 窗口可见性控制 + 切换期间暂停
- 实现当前输入标记：活跃输入按钮显示绿色脉冲圆点 + "当前"标签
- 实现切换后验证：macOS 等待 500ms 后重新读取输入状态，检测无信号端口
- 实现并发切换保护：useRef 互斥锁防止快速连点导致 DDC 总线冲突
- 动态输入源列表：自动添加当前输入值到支持列表（即使不在预定义列表中）
- 检测/空状态添加"重新检测"按钮
- 切换完成后使用静默刷新（不显示 loading 骨架屏）

## Capabilities

### New Capabilities

- `ui-polish`: 前端界面美化与交互优化（加载态、暗色模式、空状态、响应式布局、编辑按钮、品牌色）
- `backend-bugfix`: 后端 bug 修复与代码清理（托盘 id、死代码、错误处理、m1ddc 打包）
- `tray-polish`: 托盘菜单完善（版本标题、自定义名称、帮助子菜单、正式化）
- `toast-notification`: 全局 Toast 通知系统（切换中/成功/警告/失败，底部定位）
- `hot-plug-detection`: 显示器热插拔检测（定时轮询 + 可见性控制）
- `switch-verification`: 切换后验证与反馈（读取实际输入状态、无信号端口警告）
- `concurrent-protection`: 并发切换保护（互斥锁防止 DDC 总线冲突）

### Modified Capabilities

- `input-switching`: 新增当前输入标记（绿色脉冲 + "当前"标签）、动态输入源列表、静默刷新

## Impact

- **前端**：`App.tsx`（Toast 系统、轮询、互斥锁）、`monitor-card.tsx`（当前输入标记）、`index.css`（品牌色、滚动条隐藏）、`index.html`（语言属性、暗色模式脚本）、新增 `types/monitor.ts`、`monitor-card-skeleton.tsx`
- **后端**：`tray.rs`（帮助子菜单、正式化）、`config.rs`（移除死代码和 tips_dismissed）、`lib.rs`（日志插件、错误处理）、`monitor.rs`（切换后验证、内置显示器过滤、动态输入列表、m1ddc sidecar）
- **打包**：`src-tauri/binaries/m1ddc-aarch64-apple-darwin`、`tauri.conf.json` 增加 `externalBin`
- **根目录**：新增 `LICENSE`
- **新增 Rust 依赖**：`open`（打开浏览器链接）
- **删除前端依赖**：`@tauri-apps/plugin-fs`、`@tauri-apps/plugin-shell`、`lucide-react`
- **删除组件**：`feature-tips.tsx`、`scroll-area.tsx`、`tooltip.tsx`、`separator.tsx`
