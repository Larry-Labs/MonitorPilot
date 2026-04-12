# monitor.rs 模块化重构设计

## 一、背景与动机

### 当前问题

`src-tauri/src/monitor.rs` 为单文件 646 行，存在以下问题：

1. **验证逻辑重复**：macOS 和 Linux/Windows 各有独立的 switch 验证 + 回滚逻辑，约 80% 代码重复
2. **验证策略不一致**：macOS 两轮验证（600ms + 1400ms），Linux/Windows 仅单次验证（600ms）
3. **职责混杂**：类型定义、输入映射、m1ddc CLI 工具链、ddc-hi crate 适配、验证逻辑全部在一个文件
4. **难以扩展**：若要调整验证策略或新增平台，需要同时修改多处

### 重构目标

- 提取通用验证框架，消除代码重复
- 统一跨平台验证策略（两轮验证）
- 按职责拆分为独立子模块
- **保持外部 API 不变**，对 `lib.rs` 和 `tray.rs` 透明

## 二、当前代码结构

```
src-tauri/src/monitor.rs (646 行)
│
├── Types (15-34)
│   InputSource, MonitorInfo, SwitchResult
│
├── Shared Utils (36-80)
│   input_name, is_known_input, supported_inputs_with_current
│
├── Constants (7-13)
│   DDC_LOCK, VCP_INPUT_SOURCE, POST_SWITCH_VERIFY_DELAY_MS, M1DDC_TIMEOUT
│
├── macOS (82-390) — #[cfg(target_os = "macos")]
│   ├── M1DDC_PATH, find_m1ddc, run_m1ddc       (84-144)
│   ├── get_monitors, parse_m1ddc_line            (146-239)
│   ├── macos_get_input                           (241-270)
│   └── switch_input (含验证+回滚)                (272-390)
│
├── Linux/Windows (392-535) — #[cfg(linux/windows)]
│   ├── get_monitors                              (394-438)
│   └── switch_input (含验证+回滚)                (440-535)
│
└── Tests (537-645)
    ├── Shared: input_name, is_known_input, supported_inputs  (541-597)
    └── macOS: parse_m1ddc_line                               (599-644)
```

### 外部依赖关系

| 文件 | 导入内容 |
|------|----------|
| `lib.rs` L7 | `use monitor::{get_monitors, switch_input, MonitorInfo, SwitchResult}` |
| `tray.rs` L9 | `use crate::monitor::{get_monitors, switch_input}` |
| `tray.rs` L139 | `&crate::monitor::MonitorInfo` |

## 三、目标结构

```
src-tauri/src/monitor/
├── mod.rs          — 模块定义 + DDC_LOCK + re-exports
├── types.rs        — InputSource, MonitorInfo, SwitchResult
├── input_map.rs    — input_name, is_known_input, supported_inputs_with_current
├── verify.rs       — DdcOps trait + verify_switch + attempt_rollback
├── macos.rs        — #[cfg(macos)] MacOsDdc + m1ddc 工具链 + get_monitors
└── desktop.rs      — #[cfg(linux/windows)] DdcHiAdapter + ddc-hi + get_monitors
```

### 各模块职责

#### `mod.rs` — 模块入口

```rust
mod types;
mod input_map;
mod verify;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod desktop;

// Re-exports: 保持外部 API 路径不变
// 仅导出外部代码实际使用的类型（InputSource 和 input_map 函数仅模块内部使用）
pub use types::{MonitorInfo, SwitchResult};

#[cfg(target_os = "macos")]
pub use macos::{get_monitors, switch_input};
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use desktop::{get_monitors, switch_input};

use std::sync::Mutex;
pub(crate) static DDC_LOCK: Mutex<()> = Mutex::new(());
```

#### `types.rs` — 数据类型

```rust
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct InputSource {
    pub value: u8,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct MonitorInfo {
    pub index: usize,
    pub model: String,
    pub current_input: Option<u8>,
    pub current_input_name: String,
    pub supported_inputs: Vec<InputSource>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SwitchResult {
    pub status: String,
    pub message: String,
}
```

#### `input_map.rs` — 输入源映射

原有 `input_name`、`is_known_input`、`supported_inputs_with_current` 函数原样迁移，仅调整 import 路径。包含对应的单元测试。

#### `verify.rs` — 通用验证框架（核心新增）

```rust
use std::time::Duration;
use super::input_map::input_name;
use super::types::SwitchResult;

/// 验证延迟：第一轮 ~0.6s，第二轮 ~2.0s（累计）
const VERIFY_DELAYS: &[u64] = &[600, 1400];
/// 回滚后验证等待
const POST_ROLLBACK_VERIFY_MS: u64 = 600;

/// DDC 读写抽象，各平台实现此 trait
pub(crate) trait DdcOps {
    fn read_input(&mut self) -> Option<u8>;
    fn write_input(&mut self, value: u8) -> Result<(), String>;
}

/// 通用多轮验证：切换命令发送后调用
pub(crate) fn verify_switch(
    target_value: u8,
    previous_input: Option<u8>,
    ops: &mut dyn DdcOps,
) -> Result<SwitchResult, String> { /* ... */ }

/// 通用回滚：验证首轮 DDC 不可达时调用
fn attempt_rollback(
    target_value: u8,
    previous_input: Option<u8>,
    ops: &mut dyn DdcOps,
) -> Result<SwitchResult, String> { /* ... */ }
```

#### `macos.rs` — macOS 专属

```rust
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use super::types::{MonitorInfo, SwitchResult};
use super::input_map::{input_name, supported_inputs_with_current};
use super::verify::{DdcOps, verify_switch};

const M1DDC_TIMEOUT: Duration = Duration::from_secs(5);
static M1DDC_PATH: OnceLock<String> = OnceLock::new();

/// m1ddc DDC 适配器
struct MacOsDdc { display_num: u32 }

impl DdcOps for MacOsDdc {
    fn read_input(&mut self) -> Option<u8> { macos_get_input(self.display_num) }
    fn write_input(&mut self, value: u8) -> Result<(), String> {
        macos_set_input(self.display_num, value)
    }
}

// 以下函数原样迁移：
// find_m1ddc, run_m1ddc, parse_m1ddc_line, macos_get_input
// 新增：macos_set_input（从 switch_input 中提取的写入逻辑）

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> { /* 原样迁移 */ }

pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<SwitchResult, String> {
    let _guard = super::DDC_LOCK.lock()
        .map_err(|_| "DDC 操作正忙，请稍后重试".to_string())?;

    let display_num = monitor_index as u32;
    let mut ops = MacOsDdc { display_num };
    let previous_input = ops.read_input();

    log::info!(
        "切换请求: 显示器 #{} | {} → {} | 当前: {}",
        display_num,
        previous_input.map(input_name).unwrap_or_else(|| "未知".to_string()),
        input_name(input_value),
        previous_input.map(|v| format!("0x{:02X}", v)).unwrap_or_else(|| "N/A".to_string())
    );

    ops.write_input(input_value)
        .map_err(|e| { log::error!("切换失败: {}", e); format!("切换失败: {}", e) })?;

    verify_switch(input_value, previous_input, &mut ops)
}
```

#### `desktop.rs` — Linux/Windows 专属

```rust
use super::types::{MonitorInfo, SwitchResult};
use super::input_map::{input_name, supported_inputs_with_current};
use super::verify::{DdcOps, verify_switch};
use ddc_hi::{Ddc, Display};

const VCP_INPUT_SOURCE: u8 = 0x60;

/// ddc-hi DDC 适配器
struct DdcHiAdapter { display: Display }

impl DdcOps for DdcHiAdapter {
    fn read_input(&mut self) -> Option<u8> {
        self.display.handle.get_vcp_feature(VCP_INPUT_SOURCE)
            .ok().map(|v| v.value() as u8)
    }
    fn write_input(&mut self, value: u8) -> Result<(), String> {
        self.display.handle.set_vcp_feature(VCP_INPUT_SOURCE, value as u16)
            .map_err(|e| format!("DDC 写入失败: {}", e))
    }
}

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> { /* 原样迁移 */ }

pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<SwitchResult, String> {
    let _guard = super::DDC_LOCK.lock()
        .map_err(|_| "DDC 操作正忙，请稍后重试".to_string())?;

    let displays = Display::enumerate();
    let display = displays.into_iter().nth(monitor_index)
        .ok_or_else(|| {
            log::error!("未找到显示器 #{}", monitor_index);
            format!("切换失败: 未找到显示器 #{}", monitor_index)
        })?;

    let mut ops = DdcHiAdapter { display };
    let previous_input = ops.read_input();

    log::info!(
        "切换请求: 显示器 #{} | {} → {} | 当前: {}",
        monitor_index,
        previous_input.map(input_name).unwrap_or_else(|| "未知".to_string()),
        input_name(input_value),
        previous_input.map(|v| format!("0x{:02X}", v)).unwrap_or_else(|| "N/A".to_string())
    );

    ops.write_input(input_value)
        .map_err(|e| {
            log::error!("切换失败: 显示器 #{} → {} | {}", monitor_index, input_name(input_value), e);
            format!("切换失败: {}", e)
        })?;

    verify_switch(input_value, previous_input, &mut ops)
}
```

## 四、行为变化分析

### 功能增强（非破坏性）

| 变更 | 影响 |
|------|------|
| Linux/Windows 验证从单次 600ms 升级为两轮 600ms+1400ms | 切换操作耗时从 ~0.6s 增加到 ~2s |
| Linux/Windows 新增首轮 DDC 不可达时的自动回滚 | 原来直接返回错误，现在会尝试恢复原输入 |

### 功能不变

| 项目 | 说明 |
|------|------|
| macOS 两轮验证逻辑 | 延迟数组、判断条件、回滚流程完全相同 |
| 外部 API 签名 | `get_monitors` 和 `switch_input` 签名不变 |
| `lib.rs`/`tray.rs` 导入路径 | 通过 `mod.rs` re-export 保持 `crate::monitor::*` 路径 |
| `DDC_LOCK` 互斥语义 | 位置从文件顶部移到 `mod.rs`，锁的行为不变 |
| 所有 20 个现有测试 | 仅移动到对应子模块，断言不变 |

### 风险点

| 风险 | 概率 | 缓解措施 |
|------|------|----------|
| `ddc-hi` Display handle 在两轮验证间失效 | 低 | `DdcHiAdapter` 持有 `Display` 所有权，生命周期覆盖整个操作 |
| Rust borrow checker 拒绝 `&mut dyn DdcOps` 模式 | 低 | trait object 模式是标准 Rust 实践，已验证可行 |
| Linux/Windows 2s 延迟影响用户体验 | 低 | 与 macOS 一致；前端已有 "正在切换" Toast 提示 |

## 五、测试计划

| 测试类型 | 位置 | 内容 |
|----------|------|------|
| 共享单元测试 | `input_map.rs` | `input_name`、`is_known_input`、`supported_inputs_with_current`（6 个） |
| macOS 解析测试 | `macos.rs` | `parse_m1ddc_line`（6 个） |
| 配置测试 | `config.rs` | 不变（8 个） |
| 前端测试 | `src/__tests__/` | 不变（37 个，mock 层不受后端重构影响） |

**验证步骤**：
1. `cargo test` — 20 个后端测试全部通过
2. `npm test` — 37 个前端测试全部通过
3. `cargo clippy` — 无新警告
4. `npx tsc --noEmit` — 类型检查通过

## 六、Review 发现的注意事项

### 6.1 macOS `write_input` 必须包含 stdout 错误解析

当前 `switch_input` 在 m1ddc 进程成功退出（exit code 0）后，仍检查 stdout 中的错误字符串：

```rust
let stdout = String::from_utf8_lossy(&output.stdout);
let trimmed = stdout.trim();
if trimmed.contains("Could not find") || trimmed.starts_with("Error") || trimmed.starts_with("error:") {
    return Err(format!("切换失败: {}", trimmed));
}
```

`MacOsDdc::write_input` 的实现**必须**包含这两段检查（exit code + stdout 文本），否则会漏检"退出码 0 但实际报错"的场景。

### 6.2 保留详细日志格式

当前 macOS `switch_input` 的日志包含前一输入的十六进制值：

```rust
log::info!(
    "切换请求: 显示器 #{} | {} → {} | 当前: {}",
    display_num,
    previous_input.map(input_name).unwrap_or_else(|| "未知".to_string()),
    input_name(input_value),
    previous_input.map(|v| format!("0x{:02X}", v)).unwrap_or_else(|| "N/A".to_string())
);
```

重构后的 `switch_input` 必须保留完整的四段日志格式，不得简化。

### 6.3 `POST_ROLLBACK_VERIFY_MS` 覆盖所有原有等待点

当前 `POST_SWITCH_VERIFY_DELAY_MS = 600` 用于：
- macOS 回滚后验证等待（L357）
- Linux/Windows 单次验证等待（L481）
- Linux/Windows 回滚后验证等待（L516）

重构后：
- 多轮验证等待由 `VERIFY_DELAYS = [600, 1400]` 覆盖（替代 L481）
- 回滚后验证等待由 `POST_ROLLBACK_VERIFY_MS = 600` 覆盖（替代 L357、L516）

实现时需逐一对照确认无遗漏。

## 七、不修改的文件

| 文件 | 原因 |
|------|------|
| `lib.rs` | 导入路径 `use monitor::*` 因 re-export 不变 |
| `tray.rs` | 同上 |
| `display_observer.rs` | 不依赖 `monitor` 模块 |
| `config.rs` | 不依赖 `monitor` 模块 |
| 所有前端文件 | 后端重构对 IPC 层透明 |
