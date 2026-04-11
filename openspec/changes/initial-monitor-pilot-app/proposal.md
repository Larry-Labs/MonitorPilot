## Why

多显示器用户经常需要在不同输入源（DP、HDMI、USB-C 等）之间切换，但目前只能通过显示器物理按键操作 OSD 菜单，体验繁琐且低效。DDC/CI 协议允许通过软件控制显示器设置，但现有的命令行工具（m1ddc、ddcutil）缺乏直观的图形界面，且不同平台需要不同的工具。MonitorPilot 旨在提供一个统一的、跨平台的图形化显示器输入源切换工具。

## What Changes

- 创建全新的跨平台桌面应用 MonitorPilot
- 基于 Tauri 2 框架（Rust 后端 + Web 前端），支持 macOS / Linux / Windows
- 通过 DDC/CI 协议与显示器通信，实现软件控制输入源切换
- 提供系统托盘（System Tray）常驻应用，一键切换
- 自动检测已连接的显示器及其支持的输入源
- 支持用户自定义输入源命名
- 单实例运行保证（不会重复启动多个实例）

## Capabilities

### New Capabilities

- `monitor-detection`: 自动检测已连接的显示器，读取显示器型号、当前输入源等信息
- `input-switching`: 通过 DDC/CI 协议切换显示器输入源（DP、HDMI、USB-C 等）
- `system-tray`: 系统托盘常驻应用，提供快速切换菜单
- `single-instance`: 单实例运行保证，重复启动时聚焦已有窗口
- `cross-platform`: 跨平台支持（macOS / Linux / Windows），统一的用户体验

### Modified Capabilities

（无，这是全新项目）

## Impact

- **技术栈**：Tauri 2 + React + TypeScript（前端）、Rust（后端 DDC/CI 通信）
- **系统依赖**：
  - macOS：需要 I/O Kit 权限访问显示器 DDC/CI
  - Linux：需要 i2c-dev 内核模块和用户组权限
  - Windows：通过 Win32 API（SetVCPFeature/GetVCPFeature）访问
- **Rust 依赖**：`ddc-hi` crate（Linux/Windows DDC/CI 库）、macOS 使用 `m1ddc` CLI
- **打包分发**：.dmg（macOS）/ .exe NSIS 安装包（Windows）/ .AppImage（Linux）
