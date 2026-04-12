# Tasks: DDC 扩展控制 & 多显示器管理（v0.2.0）

## 1. 后端：DdcOps trait 扩展

- [ ] 1.1 扩展 `DdcOps` trait，新增 `read_vcp(code: u8) -> Option<u16>` 和 `write_vcp(code: u8, value: u16) -> Result<(), String>`
- [ ] 1.2 `MacOsDdc` 适配器实现 `read_vcp` / `write_vcp`（通过 `m1ddc get/set` 命令）
- [ ] 1.3 `DdcHiAdapter` 适配器实现 `read_vcp` / `write_vcp`（通过 `ddc-hi` crate API）
- [ ] 1.4 单元测试：VCP 读写的 mock 测试

## 2. 后端：DDC 控制命令

- [ ] 2.1 扩展 `MonitorInfo` 结构体，新增 `brightness`、`contrast`、`volume`、`power_mode` 字段
- [ ] 2.2 `get_monitors` 中读取 DDC 控制参数（VCP 0x10/0x12/0x62/0xD6）
- [ ] 2.3 新增 Tauri 命令 `cmd_set_brightness(monitor_index, value)`
- [ ] 2.4 新增 Tauri 命令 `cmd_set_contrast(monitor_index, value)`
- [ ] 2.5 新增 Tauri 命令 `cmd_set_volume(monitor_index, value)`
- [ ] 2.6 新增 Tauri 命令 `cmd_set_power_mode(monitor_index, mode)`
- [ ] 2.7 DDC 控制参数的读取策略：窗口可见时读取，不持续轮询

## 3. 前端：DDC 控制面板 UI

- [ ] 3.1 MonitorCard 新增折叠面板（默认收起），包含亮度/对比度/音量滑块
- [ ] 3.2 滑块组件实现 300ms 防抖
- [ ] 3.3 不支持的参数（None）自动隐藏对应控件
- [ ] 3.4 电源模式按钮（开启/待机切换）
- [ ] 3.5 前端测试：滑块渲染、防抖、隐藏逻辑

## 4. 后端：预设管理

- [ ] 4.1 定义 `InputPreset` 数据结构
- [ ] 4.2 扩展 `AppConfig`，新增 `presets` 和 `monitor_order` 字段
- [ ] 4.3 新增 Tauri 命令 `cmd_save_preset(name)`：读取当前所有显示器输入，保存为预设
- [ ] 4.4 新增 Tauri 命令 `cmd_apply_preset(preset_id)`：按顺序切换所有显示器
- [ ] 4.5 新增 Tauri 命令 `cmd_delete_preset(preset_id)`
- [ ] 4.6 新增 Tauri 命令 `cmd_list_presets()`
- [ ] 4.7 预设执行的错误处理：部分失败不中断，汇总结果
- [ ] 4.8 单元测试：预设 CRUD、批量切换逻辑

## 5. 前端：预设管理 UI

- [ ] 5.1 预设管理页面/面板（列表 + 新建/删除）
- [ ] 5.2 "保存当前配置"按钮 + 名称输入
- [ ] 5.3 "应用预设"按钮 + 批量切换进度 Toast
- [ ] 5.4 预设中缺失显示器的提示
- [ ] 5.5 前端测试

## 6. 显示器排序

- [ ] 6.1 前端：显示器卡片排序 UI（上移/下移按钮或拖拽）
- [ ] 6.2 后端：`cmd_set_monitor_order(order: Vec<usize>)` 命令
- [ ] 6.3 `get_monitors` 返回按 `monitor_order` 排序的列表
- [ ] 6.4 新显示器追加到末尾
- [ ] 6.5 测试

## 7. 托盘集成

- [ ] 7.1 托盘菜单新增"预设"子菜单
- [ ] 7.2 动态加载预设列表到菜单项
- [ ] 7.3 托盘点击预设 → 执行批量切换 → emit 事件通知前端
- [ ] 7.4 测试

## 8. 文档与发布

- [ ] 8.1 更新 docs/switching-logic.md（DDC 控制参数部分）
- [ ] 8.2 更新 README.md（新功能说明）
- [ ] 8.3 更新 CHANGELOG.md
- [ ] 8.4 版本号更新为 0.2.0
- [ ] 8.5 编译测试全平台
- [ ] 8.6 提交并推送到 GitHub
