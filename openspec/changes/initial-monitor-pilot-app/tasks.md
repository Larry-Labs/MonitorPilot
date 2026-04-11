## 1. 项目脚手架搭建

- [x] 1.1 初始化 Tauri 2 项目（React + TypeScript + Vite）
- [x] 1.2 配置 Tailwind CSS 4 和 shadcn/ui
- [x] 1.3 添加 Rust 依赖：`ddc-hi`、`serde`、`serde_json` 到 `Cargo.toml`
- [x] 1.4 配置 Tauri 权限：core:default（配置文件通过 Rust std::fs 读写，不使用 Tauri fs 插件）
- [x] 1.5 创建项目 README.md，说明构建和运行方式

## 2. Rust 后端：DDC/CI 通信层

- [x] 2.1 实现 `MonitorInfo` 数据结构（显示器型号、ID、支持的输入源列表、当前输入源）
- [x] 2.2 实现 `get_monitors()` 函数：macOS 使用 m1ddc CLI，Linux/Windows 使用 ddc-hi
- [x] 2.3 实现 `switch_input()` 函数：切换指定显示器到指定输入源
- [x] 2.4 添加错误处理：权限不足、显示器不可达、DDC/CI 未启用等场景
- [ ] 2.5 实现精确的输入源检测：解析显示器实际支持的输入源（替代当前的固定列表）（需连接显示器后调试）— 当前采用预置列表 + 当前输入动态并入策略

## 3. Rust 后端：配置管理

- [x] 3.1 定义配置数据结构（`AppConfig`：自定义输入名称映射）
- [x] 3.2 实现配置文件的读取和写入（JSON 格式，存储在 `app_data_dir`）
- [x] 3.3 实现首次启动时的默认配置生成

## 4. Tauri 命令注册（IPC 桥接）

- [x] 4.1 注册 `cmd_get_monitors` 命令：返回所有检测到的显示器信息
- [x] 4.2 注册 `cmd_switch_input` 命令：切换指定显示器的输入源
- [x] 4.3 注册 `cmd_get_config` / `cmd_save_config` 命令

## 5. 系统托盘实现

- [x] 5.1 创建系统托盘图标和基础托盘菜单
- [x] 5.2 动态生成托盘菜单：显示器列表 → 输入源子菜单（标记当前激活项）
- [x] 5.3 实现托盘菜单点击事件：一键切换输入源
- [x] 5.4 添加 "Settings"、"Refresh" 和 "Quit" 菜单项
- [x] 5.5 实现切换后托盘菜单的实时更新

## 6. 前端 UI：设置窗口

- [x] 6.1 搭建设置窗口的基础布局（shadcn/ui + Tailwind CSS）
- [x] 6.2 实现显示器列表页面：显示检测到的显示器、当前输入源、切换按钮
- [x] 6.3 实现输入源自定义命名功能：双击按钮编辑名称，保存到配置文件

## 7. 单实例保证

- [x] 7.1 添加 Tauri single-instance 插件依赖
- [x] 7.2 实现单实例检测：如果已有实例运行，将焦点切换到已有实例并退出新实例

## 8. 移除快捷键功能

- [x] 8.1 移除全局快捷键相关代码（config.rs 中的 HotkeyBinding、lib.rs 中的注册逻辑）
- [x] 8.2 移除前端快捷键配置组件（hotkey-config.tsx）
- [x] 8.3 移除 global-shortcut 插件依赖
- [x] 8.4 清理 capabilities 中的 global-shortcut 权限

## 9. 跨平台适配

- [ ] 9.1 实现 Linux 权限检测与 i2c-dev 模块加载引导（需 Linux 环境测试）— 当前通过 README 说明手动配置步骤

## 10. 打包与分发

- [x] 10.1 配置 Tauri 打包：macOS (.dmg)、Windows (.exe NSIS 安装包)、Linux (.AppImage)
- [x] 10.2 配置 GitHub Actions CI/CD 自动构建三平台安装包
