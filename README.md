# MonitorPilot

跨平台显示器输入源切换工具。通过 DDC/CI 协议控制显示器，无需按物理按键即可切换 DP / HDMI / USB-C 等输入源。

## 功能

- 自动检测 DDC/CI 兼容显示器及其支持的输入源
- 系统托盘常驻，右键快速切换
- 自定义输入源名称（如 "MacBook" 代替 "DP-1"）
- 单实例运行保证（不会重复启动多个实例）
- 热插拔检测：macOS 原生实时检测（CoreGraphics）+ 3 秒轮询兜底
- 切换后验证：自动读回输入状态，检测无信号端口并自动回滚到上一端口
- 并发保护：DDC 串行锁防止快速操作导致总线冲突
- Toast 通知：切换中/成功/警告/失败四态反馈
- 暗色模式：自动跟随系统偏好
- 支持 macOS / Linux / Windows
- 极致轻量

## 技术栈

- **后端**：Rust + [m1ddc](https://github.com/waydabber/m1ddc)（macOS）/ [ddc-hi](https://crates.io/crates/ddc-hi)（Linux/Windows）
- **前端**：React + TypeScript + [shadcn/ui](https://ui.shadcn.com/) + Tailwind CSS
- **框架**：[Tauri 2](https://v2.tauri.app/)
- **原生检测**：CoreGraphics（macOS 显示器变化回调）

## 前置条件

- [Node.js](https://nodejs.org/) >= 20
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- 显示器需开启 DDC/CI（在 OSD 菜单中设置）

### macOS

macOS 版本已内置 m1ddc，无需额外安装。

### Linux 额外要求

```bash
sudo modprobe i2c-dev
sudo usermod -aG i2c $USER
```

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式（前端 + Rust 后端热重载）
npm run tauri dev

# 仅前端开发（Vite）
npm run dev

# 构建生产版本
npm run tauri build
```

## 测试

```bash
# 前端测试（Vitest + React Testing Library）
npm test

# 前端测试（监听模式）
npm run test:watch

# 后端测试（Rust）
cd src-tauri && cargo test
```

前端共 37 个测试（App 生命周期/切换/轮询/重命名、MonitorCard 交互、类型定义），后端共 20 个测试（输入映射、配置持久化、m1ddc 解析）。CI 在每次 push/PR 时自动运行测试、类型检查和 clippy。

## 架构

```
┌─────────────────────────────────────────────┐
│  Frontend (React + TypeScript)              │
│  App.tsx → MonitorCard → Toast / Loading    │
│  listen("display-changed") ← Tauri Event   │
│  setInterval(silentRefresh, 3000)           │
├─────────────────────────────────────────────┤
│  Tauri IPC                                  │
│  cmd_get_monitors / cmd_switch_input        │
├─────────────────────────────────────────────┤
│  Backend (Rust)                             │
│  monitor.rs   — DDC/CI 读写 + 切换验证     │
│  config.rs    — 配置持久化                  │
│  tray.rs      — 系统托盘菜单               │
│  display_observer.rs — 原生显示器事件       │
├─────────────────────────────────────────────┤
│  Platform Layer                             │
│  macOS: m1ddc + CoreGraphics callback       │
│  Linux/Win: ddc-hi + polling                │
└─────────────────────────────────────────────┘
```

### 热插拔检测

| 平台 | 检测方式 | 延迟 |
|---|---|---|
| macOS | CoreGraphics `CGDisplayRegisterReconfigurationCallback` | < 1 秒 |
| Windows / Linux | 3 秒轮询 | 3-6 秒 |

macOS 原生回调覆盖物理线缆插拔；显示器休眠/唤醒等状态变化由轮询兜底。

## 分发格式

| 平台 | 格式 |
|------|------|
| macOS | `.dmg` |
| Windows | `.exe`（NSIS 安装包） |
| Linux | `.AppImage` |

## 许可证

MIT
