# MonitorPilot

跨平台显示器输入源切换工具。通过 DDC/CI 协议控制显示器，无需按物理按键即可切换 DP / HDMI / USB-C 等输入源。

## 功能

- 自动检测 DDC/CI 兼容显示器及其支持的输入源
- 系统托盘常驻，右键快速切换
- 自定义输入源名称（如 "MacBook" 代替 "DP-1"）
- 单实例运行保证（不会重复启动多个实例）
- 支持 macOS / Linux / Windows
- 极致轻量（macOS .dmg 仅 3.7MB）

## 技术栈

- **后端**：Rust + [m1ddc](https://github.com/waydabber/m1ddc)（macOS）/ [ddc-hi](https://crates.io/crates/ddc-hi)（Linux/Windows）
- **前端**：React + TypeScript + [shadcn/ui](https://ui.shadcn.com/) + Tailwind CSS
- **框架**：[Tauri 2](https://v2.tauri.app/)

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

# 构建生产版本
npm run tauri build
```

## 分发格式

| 平台 | 格式 |
|------|------|
| macOS | `.dmg` |
| Windows | `.exe`（绿色免安装） |
| Linux | `.AppImage` |

## 许可证

MIT
