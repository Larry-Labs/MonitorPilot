## REMOVED

此能力已在开发过程中移除。

**原因**：用户明确表示不需要全局快捷键功能。作为系统托盘应用，通过托盘菜单切换已足够便捷。

**替代方案**：系统托盘右键菜单 + 设置窗口内的一键切换按钮。

相关清理已完成：
- 移除 `tauri-plugin-global-shortcut` 依赖
- 移除前端 `hotkey-config.tsx` 组件
- 移除后端 `HotkeyBinding` 数据结构和注册逻辑
- 移除 capabilities 中的 `global-shortcut` 权限
