## Context

MonitorPilot v0.1.0 已完成核心功能（显示器检测、输入切换、系统托盘、单实例），但代码 review 发现后端存在影响正确性的 bug（托盘 id 不匹配、错误处理缺失），前端 UI 仅为功能验证级别，缺乏加载反馈、暗色模式、无障碍支持等产品级体验。

## Goals / Non-Goals

**Goals:**

- 修复所有已知后端 bug，确保托盘菜单正确刷新
- 消除死代码和类型重复，提升代码可维护性
- 将前端 UI 从原型级提升至产品级（加载态、暗色模式、空状态引导、错误提示、响应式）
- 添加 LICENSE 文件，完善开源合规
- 确保 lockfile 纳入版本控制

**Non-Goals:**

- 不扩展 DDC/CI 能力边界（如亮度调节、分屏控制等）
- 不重构 Rust 后端架构
- 不做国际化/多语言支持

> 注：测试框架已在本 change 后期引入（Vitest + React Testing Library 用于前端，Rust 内置 `#[test]` 用于后端），详见 `vite.config.ts` 和 `src-tauri/src/monitor.rs`、`config.rs` 中的测试模块。

## Decisions

### 1. 暗色模式：跟随系统偏好

**选择**：使用 CSS `prefers-color-scheme` 媒体查询自动跟随系统，不提供手动切换开关

**理由**：
- 作为工具类应用，跟随系统最符合用户预期
- 实现简单，无需额外状态管理
- shadcn/ui 主题变量已在 `index.css` 中定义好 `:root` 和 `.dark`

### 2. 加载状态：骨架屏

**选择**：使用 Skeleton 占位符代替 Spinner

**理由**：
- 骨架屏能给用户预期的布局感知，体验优于单一加载图标
- shadcn/ui 有现成的 Skeleton 组件
- MonitorPilot 加载时间极短（本地 DDC/CI），骨架屏闪现即过

### 3. 空状态：结构化引导

**选择**：图标 + 描述 + 操作建议的三段式空状态

**理由**：
- 替代当前 emoji + 纯文本，与 shadcn 设计语言一致
- 帮助用户理解为何没有检测到显示器并指引下一步

### 4. 类型共享：独立 types 文件

**选择**：将 `InputSource`/`MonitorInfo` 提取到 `src/types/monitor.ts`

**理由**：
- 消除 `App.tsx` 与 `monitor-card.tsx` 的类型重复
- 后续如有更多组件使用这些类型，只需维护一处

### 5. 编辑交互：显式编辑按钮

**选择**：为每个输入源按钮添加小型编辑图标，替代仅靠双击触发重命名

**理由**：
- 双击在触摸设备和键盘操作中不可达
- 显式按钮符合可发现性原则

### 6. Toast 通知：底部定位 + 四态区分

**选择**：自定义 Toast 组件，固定在 footer 上方，区分 switching/success/warning/error 四种状态

**理由**：
- Alert 组件用于检测异常（持久展示），Toast 用于操作反馈（短暂展示）
- 底部定位不遮挡主内容区域的显示器卡片
- 四种状态（蓝紫/绿/琥珀/红）颜色编码让用户快速识别结果
- useRef 管理 timer 防止内存泄漏

### 7. 热插拔检测：原生事件 + 轮询兜底

**选择**：macOS 使用 CoreGraphics 原生回调实现实时检测，3 秒轮询作为跨平台兜底方案

**架构**：
- **macOS**：`CGDisplayRegisterReconfigurationCallback` 监听 display add/remove 事件（`kCGDisplayAddFlag` / `kCGDisplayRemoveFlag`），触发后通过 Tauri `emit("display-changed")` 通知前端立即刷新。延迟 <1 秒
- **Windows/Linux**：暂无原生监听，依赖 3 秒轮询
- **所有平台**：3 秒静默轮询 `cmd_get_monitors`，窗口不可见时暂停（`document.visibilitychange`），切换操作期间自动暂停

**备选方案（已弃用）**：
- 纯轮询（5 秒）：延迟 3-6 秒，体验差
- 第三方 crate（`display-config`）：仅支持 macOS/Windows，且引入外部依赖
- 用户手动刷新：无额外开销但体验差

**理由**：
- CoreGraphics 回调是 macOS 原生 API，零额外依赖，注册后自动集成到 Tauri 的 CFRunLoop
- 原生回调只覆盖物理插拔，不覆盖显示器休眠/唤醒，轮询补充这部分场景
- `OnceLock<AppHandle>` 保证线程安全且仅注册一次
- 前端通过 `listen("display-changed")` 监听后端事件，与轮询互补互不冲突

### 8. 并发切换保护：useRef 互斥锁

**选择**：使用 `useRef<boolean>` 作为同步互斥锁

**理由**：
- React 的 `useState` 更新是异步的，快速连点时第二次 click 可能在第一次 `setSwitching` 生效前执行
- `useRef` 是同步更新的，可作为可靠的并发门控
- DDC/CI 是串行总线协议，并发发送命令会导致通信失败

### 9. 切换后验证：多轮等待 + 读回 + 前端静默修正

**选择**：全平台统一两轮验证（600ms + 1400ms，总计约 2s）。前端采用乐观更新 + 轮询静默修正。

**理由**：
- DDC/CI `set` 命令可能成功发送但显示器未实际切换（如目标端口无信号）
- 两轮验证捕获大部分固件立即拒绝和短延迟回退场景
- 通过 `DdcOps` trait 抽象层，macOS 和 Linux/Windows 共享同一套验证逻辑
- 验证结果分三种：完全成功（success）、目标端口无信号（warning）、DDC 通信中断（error + 尝试回滚）
- 前端收到 success 后乐观更新 UI，若后续轮询发现显示器因无信号自动回退，**静默修正按钮状态，不弹额外 Toast**
- 显示器自动回退的目标由固件决定（通常是第一个有信号端口），应用不发送冗余 DDC 命令避免屏幕闪烁
- 后端返回结构化 `SwitchResult { status, message }`，前端按 status 字段选择提示类型

### 10. 后端 DDC 操作互斥锁

**选择**：在 `monitor.rs` 中使用 `static DDC_LOCK: Mutex<()>`，在 `switch_input()` 函数内部获取锁

**理由**：
- 锁在模块入口函数内部获取，确保所有调用方（前端 IPC、托盘菜单）自动受保护
- 无需调用方记住加锁，消除了遗漏风险（之前锁在 `lib.rs` 中，托盘路径绕过了锁）
- `Mutex` 轻量且语义清晰

### 11. MonitorCard 性能优化：React.memo + useCallback

**选择**：`MonitorCard` 使用 `React.memo` 包装，父组件事件处理使用 `useCallback` + `useRef`

**理由**：
- 多显示器场景下，切换操作只应影响操作中的卡片，其余不应重渲染
- `handleRename` 通过 `customNamesRef` 避免对频繁变化的 `customNames` state 产生闭包依赖
- 配合 `useCallback` 使 `memo` 的浅比较生效

## Risks / Trade-offs

- **[暗色模式闪烁]** → 首次加载时可能出现白屏到暗色的闪烁。**缓解**：在 `index.html` 中引入外部 `theme.js` 脚本，提前设置 `dark` 类。
- **[Skeleton 组件闪烁]** → 加载极快时骨架屏一闪而过。**缓解**：设置最小显示时间 200ms 或接受快速闪烁（优于无反馈）。
- **[托盘 id 修复]** → 需确认 Tauri Rust API 中 `TrayIconBuilder` 的 id 设置方法名。**缓解**：查阅 Tauri 2 文档确认。
- **[轮询精度]** → 3 秒轮询间隔意味着非原生平台热插拔后最多等 3 秒 UI 才更新。macOS 原生检测延迟 <1 秒。
- **[切换后验证延迟]** → 600ms 等待增加了切换操作的响应时间。**缓解**：已有 Toast "正在切换" 提示，用户不会感到操作无响应。
- **[DDC 总线冲突]** → 轮询和切换同时操作可能导致总线错误。**缓解**：切换期间暂停轮询；前端互斥锁防止并发切换；后端 DDC_LOCK 作为最终保障。
- **[CSP 限制]** → 严格的 CSP 策略可能阻止第三方资源加载。**缓解**：已配置 `style-src 'unsafe-inline'` 兼容 Tailwind 内联样式，仅允许 `self` 和 `asset:` 协议。
