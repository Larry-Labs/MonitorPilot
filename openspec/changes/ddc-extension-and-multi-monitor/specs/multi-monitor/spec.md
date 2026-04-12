# Capability: 多显示器管理

## ADDED Requirements

### Requirement: 显示器排序

用户能自定义显示器在 UI 中的排列顺序。

#### Scenario: 调整显示器顺序

- **WHEN** 用户在设置页中调整显示器顺序（上移/下移）
- **THEN** 显示器在 UI 中按新顺序排列
- **THEN** 新顺序持久化到 config.json 的 `monitor_order` 字段

#### Scenario: 新显示器插入

- **WHEN** 检测到新的显示器（不在 `monitor_order` 中）
- **THEN** 新显示器追加到列表末尾

#### Scenario: 已排序的显示器拔出

- **WHEN** 已排序的显示器断开连接
- **THEN** 该显示器从 UI 中消失
- **THEN** `monitor_order` 配置保留（重新连接时恢复位置）

### Requirement: 输入预设管理

用户能保存、加载、编辑、删除多显示器输入配置预设。

#### Scenario: 保存当前配置为预设

- **WHEN** 用户点击"保存为预设"按钮
- **THEN** 系统读取所有显示器的当前输入源
- **THEN** 用户输入预设名称
- **THEN** 预设保存到 config.json

#### Scenario: 加载预设（一键切换）

- **WHEN** 用户选择一个预设并点击"应用"
- **THEN** 系统按顺序切换每台显示器到预设指定的输入源
- **THEN** 每台显示器使用现有的即时乐观更新 + 后端验证流程
- **THEN** 预设中缺失的显示器（已拔出）跳过，显示提示

#### Scenario: 删除预设

- **WHEN** 用户选择预设并点击"删除"
- **THEN** 确认对话框
- **THEN** 从 config.json 中移除该预设

### Requirement: 托盘预设快速切换

用户能从系统托盘菜单直接应用预设。

#### Scenario: 托盘菜单显示预设

- **WHEN** 用户右键/左键点击托盘图标
- **THEN** 菜单中显示"预设"子菜单
- **THEN** 子菜单列出所有已保存的预设

#### Scenario: 托盘应用预设

- **WHEN** 用户点击托盘菜单中的预设名称
- **THEN** 系统执行该预设的批量切换
- **THEN** 前端接收事件并同步 UI 状态

### Requirement: 批量切换进度反馈

#### Scenario: 多台显示器批量切换

- **WHEN** 用户应用包含多台显示器的预设
- **THEN** Toast 显示"正在应用预设：{name}（1/N）..."
- **THEN** 每台显示器切换完成后更新进度
- **THEN** 全部完成后显示"预设 {name} 已应用"

#### Scenario: 部分显示器切换失败

- **WHEN** 预设中某台显示器切换失败
- **THEN** 继续切换其余显示器（不中断）
- **THEN** 完成后 Toast 汇总："{N} 台成功，{M} 台失败"
