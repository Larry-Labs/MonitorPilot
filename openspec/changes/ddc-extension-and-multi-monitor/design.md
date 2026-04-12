# Design: DDC 扩展控制 & 多显示器管理（v0.2.0）

## Context

v0.1.0 已完成核心架构：Tauri 2 + DdcOps trait 抽象 + 模块化后端 + 即时乐观更新前端。v0.2.0 在此基础上扩展 DDC 控制能力并优化多显示器管理体验。

## Goals

1. 用户能通过 UI 滑块调节显示器亮度、对比度、音量
2. 用户能通过按钮控制显示器电源状态
3. 用户能保存、加载、删除输入预设
4. 用户能一键将所有显示器切换到预设配置
5. 用户能自定义显示器在 UI 中的排列顺序

## Non-Goals

- 不做 DDC 能力自动探测（VCP Capabilities String 解析复杂度高，留给 v0.3.0）
- 不做 OSD 替代（不控制色温、gamma 等高级参数）
- 不做跨设备同步（如通过网络同步预设）

## Architecture Decisions

### 1. DDC 扩展控制

**扩展 DdcOps trait**：

```rust
pub(crate) trait DdcOps {
    fn read_input(&mut self) -> Option<u8>;
    fn write_input(&mut self, value: u8) -> Result<(), String>;
    fn read_vcp(&mut self, code: u8) -> Option<u16>;
    fn write_vcp(&mut self, code: u8, value: u16) -> Result<(), String>;
}
```

- `read_vcp` / `write_vcp` 为通用 VCP 读写接口
- 亮度 / 对比度 / 音量的具体 VCP Code 在应用层定义，不硬编码在 trait 中
- macOS 通过 `m1ddc set/get` 命令实现
- Linux/Windows 通过 `ddc-hi` crate 的 `get_vcp_feature` / `set_vcp_feature` 实现

**MonitorInfo 扩展**：

```rust
pub struct MonitorInfo {
    pub index: usize,
    pub model: String,
    pub current_input: Option<u8>,
    pub current_input_name: String,
    pub supported_inputs: Vec<InputSource>,
    // v0.2.0 新增
    pub brightness: Option<u16>,     // VCP 0x10, 0-100
    pub contrast: Option<u16>,       // VCP 0x12, 0-100
    pub volume: Option<u16>,         // VCP 0x62, 0-100
    pub power_mode: Option<u8>,      // VCP 0xD6
}
```

**前端 UI**：
- MonitorCard 新增折叠面板，包含亮度/对比度/音量滑块
- 滑块使用防抖（debounce 300ms）避免频繁 DDC 写入
- 电源按钮独立显示（开/待机切换）
- 不支持的参数（read 返回 None）隐藏对应控件

### 2. 多显示器管理

**预设数据结构**：

```typescript
interface InputPreset {
  id: string;           // UUID
  name: string;         // 用户自定义名称
  monitors: {
    index: number;      // 显示器索引
    model: string;      // 显示器型号（用于匹配）
    input_value: number; // 目标输入源
  }[];
}
```

**预设存储**：扩展 `config.json`

```json
{
  "custom_names": { ... },
  "monitor_order": [2, 1],
  "presets": [
    {
      "id": "uuid-1",
      "name": "工作模式",
      "monitors": [
        { "index": 2, "model": "R27qe Gen2", "input_value": 15 }
      ]
    }
  ]
}
```

**预设执行**：
- 遍历预设中的显示器列表，按顺序调用 `switch_input`
- 每台显示器切换使用现有的 DDC_LOCK 串行化
- 使用现有的即时乐观更新 + 后端验证流程

**排序**：
- UI 拖拽排序或上/下按钮调整
- 持久化到 config.json 的 `monitor_order` 字段
- 启动时按 `monitor_order` 排列，新发现的显示器追加到末尾

**托盘集成**：
- 托盘菜单新增"预设"子菜单
- 每个预设作为菜单项
- 点击后批量切换所有显示器

### 3. DDC 读取频率控制

亮度/对比度等参数不需要像 input source 那样频繁轮询：
- input source：每 3 秒轮询（保持不变）
- 亮度/对比度/音量：窗口打开时读取一次 + 手动刷新
- 电源模式：与 input source 同频轮询

理由：亮度等参数很少从外部变化，且频繁 DDC 读取可能影响显示器响应速度。

### 4. 错误处理

- 不支持的 VCP code：显示器返回错误或 None → 隐藏对应控件
- DDC 写入失败：显示错误 Toast，UI 不更新（不做乐观更新）
- 预设中的显示器不存在：跳过该显示器，继续执行其余
