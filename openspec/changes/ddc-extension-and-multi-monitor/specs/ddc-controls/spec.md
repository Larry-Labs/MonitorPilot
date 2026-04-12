# Capability: DDC 扩展控制

## ADDED Requirements

### Requirement: 亮度调节

用户能通过 UI 滑块调节显示器亮度。

#### Scenario: 成功调节亮度

- **WHEN** 用户拖动亮度滑块到目标值
- **THEN** 系统通过 DDC VCP 0x10 发送 set 命令
- **THEN** 滑块防抖 300ms 后发送（避免频繁写入）
- **THEN** 显示器亮度变化

#### Scenario: 显示器不支持亮度调节

- **WHEN** DDC 读取 VCP 0x10 返回 None 或错误
- **THEN** 亮度滑块不显示

### Requirement: 对比度调节

用户能通过 UI 滑块调节显示器对比度。

#### Scenario: 成功调节对比度

- **WHEN** 用户拖动对比度滑块到目标值
- **THEN** 系统通过 DDC VCP 0x12 发送 set 命令
- **THEN** 滑块防抖 300ms 后发送
- **THEN** 显示器对比度变化

#### Scenario: 显示器不支持对比度调节

- **WHEN** DDC 读取 VCP 0x12 返回 None 或错误
- **THEN** 对比度滑块不显示

### Requirement: 音量调节

用户能通过 UI 滑块调节显示器内置扬声器音量（部分显示器支持）。

#### Scenario: 成功调节音量

- **WHEN** 用户拖动音量滑块到目标值
- **THEN** 系统通过 DDC VCP 0x62 发送 set 命令
- **THEN** 滑块防抖 300ms 后发送
- **THEN** 显示器音量变化

#### Scenario: 显示器不支持音量调节

- **WHEN** DDC 读取 VCP 0x62 返回 None 或错误
- **THEN** 音量滑块不显示

### Requirement: 电源模式控制

用户能通过 UI 按钮控制显示器电源状态。

#### Scenario: 切换到待机模式

- **WHEN** 用户点击"待机"按钮
- **THEN** 系统通过 DDC VCP 0xD6 发送 value=0x04（待机）
- **THEN** 显示器进入待机

#### Scenario: 从待机唤醒

- **WHEN** 用户点击"唤醒"按钮
- **THEN** 系统通过 DDC VCP 0xD6 发送 value=0x01（开启）
- **THEN** 显示器唤醒

#### Scenario: 显示器不支持电源控制

- **WHEN** DDC 读取 VCP 0xD6 返回 None 或错误
- **THEN** 电源控制按钮不显示

### Requirement: DDC 控制参数读取策略

- **WHEN** 应用窗口从隐藏变为可见
- **THEN** 读取一次所有 DDC 控制参数（亮度/对比度/音量/电源）
- **THEN** 不在后台持续轮询这些参数（减少 DDC 总线负载）

> **实现说明**：input source 保持 3s 轮询不变；DDC 控制参数仅在窗口可见时按需读取。
