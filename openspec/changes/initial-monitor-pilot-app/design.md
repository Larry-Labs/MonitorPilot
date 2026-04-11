## Context

用户拥有多台设备（如 MacBook + Ubuntu 主机）共用一台显示器的场景很常见。目前切换输入源依赖显示器物理按键，操作繁琐。DDC/CI（Display Data Channel Command Interface）是 VESA 定义的标准协议，允许主机通过 I2C 总线向显示器发送控制命令（VCP Code 0x60 控制输入源）。

当前生态中缺少一个统一的、带图形界面的跨平台 DDC/CI 输入切换工具。命令行工具（m1ddc、ddcutil、ControlMyMonitor）虽然可用，但对普通用户不友好。

## Goals / Non-Goals

**Goals:**

- 提供跨平台（macOS / Linux / Windows）的图形化输入源切换工具
- 系统托盘常驻，启动后即可使用
- 自动检测显示器和可用输入源
- 用户可自定义输入源名称（如 "MacBook" 代替 "DisplayPort-1"）
- 单实例运行保证，不会重复启动多个实例
- 安装包轻量（实测 macOS .dmg 仅 3.7MB）

**Non-Goals:**

- 不做显示器的完整 DDC/CI 控制面板（亮度、对比度等由其他工具处理）
- 不做 KVM 功能（不切换键盘/鼠标，只切换显示器输入）
- 不做远程控制（仅本机操作）
- 不做 PIP/PBP 分屏模式控制
- V1 不做显示器预设配置的保存/加载

## Decisions

### 1. 框架选择：Tauri 2

**选择**：Tauri 2（Rust + Web 前端）

**备选方案**：
- Electron：成熟但体积大（100MB+），资源占用高
- Python + PyQt：开发快但打包分发复杂，需要 Python 运行时
- Flutter Desktop：UI 美观但 DDC/CI FFI 桥接复杂
- 原生开发（Swift + GTK + WinUI）：最佳性能但需维护三套代码

**理由**：
- Tauri 2 打包后仅 5-15MB，资源占用极低
- Rust 后端可原生调用 DDC/CI 库（Linux/Windows），macOS 通过 bundled m1ddc sidecar 调用
- Web 前端（React + TypeScript）开发效率高，UI 设计灵活
- Tauri 2 内置 system tray 和单实例插件支持
- 一套代码编译三平台

### 2. DDC/CI 通信层：平台差异化方案

**选择**：macOS 使用 `m1ddc` CLI，Linux/Windows 使用 `ddc-hi` crate

**背景**：`ddc-hi` 的 macOS 后端 (`ddc-macos`) 使用 `IODisplayConnect` 接口，但此接口在 Apple Silicon Mac 上不可用。`m1ddc` 使用 Apple 未公开的 `IOAVService` API，是目前 macOS（尤其是 Apple Silicon）上最可靠的 DDC/CI 工具。

**方案**：
- macOS：通过 `std::process::Command` 调用 `m1ddc` CLI（条件编译 `#[cfg(target_os = "macos")]`）
- Linux：`ddc-hi` → i2c-dev
- Windows：`ddc-hi` → Win32 Monitor Configuration API

**理由**：
- 在 Apple Silicon Mac 上经实际验证可用
- Linux/Windows 仍保持零外部依赖
- 条件编译隔离平台差异，代码清晰

### 3. 前端技术：React + shadcn/ui

**选择**：React + TypeScript + Tailwind CSS + shadcn/ui

**备选方案**：
- Svelte 5 + Skeleton UI：极致轻量但生态较小
- Vue 3 + Naive UI：中文社区强但 Tauri 模板较少
- Solid.js：性能极致但组件库匮乏

**理由**：
- React 生态最成熟，组件库丰富
- shadcn/ui 基于 Radix UI，设计精美（Linear/Raycast 风格），暗色/亮色主题开箱即用
- shadcn/ui 是复制到项目中的代码而非依赖包，完全可控不受上游升级影响
- TypeScript 提供类型安全
- Tailwind CSS 实现快速美观的 UI 定制
- Tauri 的 `@tauri-apps/api` 提供完整的前端 ↔ Rust 通信桥接

### 4. 数据持久化：JSON 配置文件

**选择**：使用本地 JSON 文件存储用户配置（输入源自定义名称）

**理由**：
- 配置数据量小，JSON 足够
- 不需要数据库，减少复杂度
- 可人工编辑，便于调试
- 使用 Tauri 的 `app_data_dir` API 获取跨平台一致的配置路径

### 5. 架构模式：命令式后端 + 响应式前端

```
┌─────────────────────────────────┐
│           System Tray           │
│    (Tauri built-in tray API)    │
└──────────────┬──────────────────┘
               │
┌──────────────▼──────────────────┐
│         React Frontend          │
│  ┌─────────┐  ┌──────────────┐  │
│  │ Monitor  │  │   Settings   │  │
│  │  List    │  │    Panel     │  │
│  └────┬─────┘  └──────┬───────┘  │
│       │               │          │
│  ─────▼───────────────▼──────    │
│  │   Tauri IPC (invoke)     │    │
│  ───────────────────────────     │
└──────────────┬──────────────────┘
               │ IPC
┌──────────────▼──────────────────┐
│         Rust Backend            │
│  ┌───────────┐ ┌─────────────┐  │
│  │  DDC/CI   │ │   Config    │  │
│  │  Service  │ │   Manager   │  │
│  └─────┬─────┘ └──────┬──────┘  │
│        │               │         │
│  ┌─────▼─────┐  ┌──────▼──────┐ │
│  │  ddc-hi   │  │  JSON File  │ │
│  │  crate    │  │  Storage    │ │
│  └───────────┘  └─────────────┘ │
└─────────────────────────────────┘
```

### 6. 单实例保证

**选择**：`tauri-plugin-single-instance`

**方案**：使用 Tauri 官方单实例插件，当用户重复启动应用时：
- 阻止第二个实例运行
- 自动显示并聚焦已有窗口

**理由**：
- 作为系统托盘常驻应用，多实例会造成混乱
- 官方插件跨平台兼容，零额外配置
- 回调机制可自定义第二次启动时的行为

### 7. 分发格式选型

| 平台 | 格式 | 说明 |
|------|------|------|
| **macOS** | `.dmg` 磁盘镜像 | 标准 macOS 分发格式，双击打开拖入 Applications |
| **Windows** | `.exe` NSIS 安装包 | 轻量安装程序，支持 per-user 安装，无需管理员权限 |
| **Linux** | `.AppImage` | 通用格式，免安装，双击直接运行，兼容所有主流发行版 |

**理由**：
- macOS 用户习惯 .dmg 安装流程
- Windows NSIS 安装包轻量，支持 per-user 安装，无需管理员权限
- AppImage 是 Linux 最通用的免安装格式，不依赖特定包管理器
- Tauri 2 内置支持以上所有格式的打包

## Risks / Trade-offs

- **[DDC/CI 兼容性]** → 并非所有显示器都支持 DDC/CI，部分显示器默认关闭。**缓解**：应用首次启动时检测并引导用户开启 DDC/CI；提供 FAQ 文档。
- **[macOS 权限]** → macOS 可能需要特殊权限才能访问 I/O Kit。**缓解**：在安装引导中说明权限需求；提供权限检查和自动请求。
- **[Linux i2c 权限]** → 需要 i2c-dev 模块和用户组权限。**缓解**：提供一键设置脚本；安装包的 post-install 脚本自动配置。
- **[输入源代码不统一]** → 不同显示器厂商的 VCP 0x60 输入源代码可能不同。**缓解**：先读取显示器支持的输入源列表（VCP capabilities string），而非硬编码。
- **[ddc-hi crate 维护风险]** → 如果该 crate 停止维护。**缓解**：ddc-hi 代码量不大，必要时可 fork 维护；底层 API 是稳定的操作系统接口。
