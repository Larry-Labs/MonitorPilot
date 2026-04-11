## 1. 项目脚手架搭建

- [x] 1.1 初始化 Tauri 2 项目（React + TypeScript + Vite）
- [x] 1.2 配置 Tailwind CSS 4 和 shadcn/ui
- [x] 1.3 添加 Rust 依赖：`ddc-hi`、`serde`、`serde_json` 到 `Cargo.toml`
- [x] 1.4 配置 Tauri 权限：system tray、global shortcut、fs（配置文件读写）
- [x] 1.5 创建项目 README.md，说明构建和运行方式

## 2. Rust 后端：DDC/CI 通信层

- [x] 2.1 实现 `MonitorInfo` 数据结构（显示器型号、ID、支持的输入源列表、当前输入源）
- [x] 2.2 实现 `get_monitors()` 函数：macOS 使用 m1ddc CLI，Linux/Windows 使用 ddc-hi
- [x] 2.3 实现 `switch_input()` 函数：切换指定显示器到指定输入源
- [x] 2.4 添加错误处理：权限不足、显示器不可达、DDC/CI 未启用等场景
- [ ] 2.5 实现精确的输入源检测：解析显示器实际支持的输入源（替代当前的固定列表）

## 3. Rust 后端：配置管理

- [ ] 3.1 定义配置数据结构（`AppConfig`：自定义输入名称映射、快捷键绑定）
- [ ] 3.2 实现配置文件的读取和写入（JSON 格式，存储在 `app_data_dir`）
- [ ] 3.3 实现首次启动时的默认配置生成

## 4. Tauri 命令注册（IPC 桥接）

- [x] 4.1 注册 `cmd_get_monitors` 命令：返回所有检测到的显示器信息
- [x] 4.2 注册 `cmd_switch_input` 命令：切换指定显示器的输入源
- [ ] 4.3 注册 `get_config` / `save_config` 命令：读写用户配置

> 注：移除了 `refresh_monitors` 独立命令 — 刷新逻辑内嵌在 `get_monitors` 中，UI 侧去掉了手动刷新按钮（自动刷新在每次切换后执行）

## 5. 系统托盘实现

- [ ] 5.1 创建系统托盘图标和基础托盘菜单
- [ ] 5.2 动态生成托盘菜单：显示器列表 → 输入源子菜单（标记当前激活项）
- [ ] 5.3 实现托盘菜单点击事件：一键切换输入源
- [ ] 5.4 添加 "Settings"、"Refresh" 和 "Quit" 菜单项
- [ ] 5.5 实现切换后托盘菜单的实时更新

## 6. 前端 UI：设置窗口

- [x] 6.1 搭建设置窗口的基础布局（shadcn/ui + Tailwind CSS）
- [x] 6.2 实现显示器列表页面：显示检测到的显示器、当前输入源、切换按钮
- [ ] 6.3 实现输入源自定义命名功能：编辑输入源的显示名称
- [ ] 6.4 实现快捷键配置页面：为每个显示器+输入源组合绑定全局快捷键
- [ ] 6.5 实现快捷键录制组件（按键捕获 → 显示组合键 → 冲突检测）

## 7. 全局快捷键

- [ ] 7.1 使用 Tauri global-shortcut 插件注册/注销全局快捷键
- [ ] 7.2 实现快捷键触发时的输入源切换逻辑
- [ ] 7.3 实现双输入源场景的 toggle 快捷键（在两个输入之间来回切换）
- [ ] 7.4 实现应用启动时自动恢复已配置的快捷键

## 8. 跨平台适配与测试

- [ ] 8.1 macOS 测试：验证 m1ddc 通信、menu bar 图标、全局快捷键
- [ ] 8.2 Linux 测试：验证 i2c-dev 通信、权限检测与引导、system tray
- [ ] 8.3 Windows 测试：验证 Win32 Monitor Configuration API、system tray、全局快捷键
- [ ] 8.4 实现 Linux 权限检测与 i2c-dev 模块加载引导

## 9. 打包与分发

- [ ] 9.1 配置 Tauri 打包：macOS (.dmg)、Windows (.exe 绿色免安装)、Linux (.AppImage)
- [ ] 9.2 设计应用图标（多尺寸，适配各平台要求）
- [ ] 9.3 配置 GitHub Actions CI/CD 自动构建三平台安装包
- [ ] 9.4 Windows 绿色版配置：确保单 .exe 文件包含所有资源，无需安装
