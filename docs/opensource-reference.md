# 开源方案调研与借鉴计划

> 调研时间：2026-04-11
> 目的：评估现有开源 DDC/CI 工具生态中可复用的稳定性和功能设计

## 一、当前技术选型评估

| 层级 | 我们的选型 | 评估 |
|------|-----------|------|
| macOS DDC 通信 | `m1ddc` CLI | ✅ 社区标准方案 |
| Linux/Windows DDC 通信 | `ddc-hi` Rust crate | ✅ Rust 生态最佳选择 |
| 桌面框架 | Tauri 2 (Rust + Web) | ✅ 跨平台，性能好 |
| 前端 UI | React + shadcn/ui | ✅ 现代化组件库 |

**结论：底层技术选型无需替换，均为各自领域的成熟方案。**

## 二、调研的开源项目

### 2.1 MonitorControl（macOS，32K+ Stars）

- **仓库**：https://github.com/MonitorControl/MonitorControl
- **技术栈**：Swift，macOS 原生
- **功能**：亮度/对比度/音量控制，OSD 通知，快捷键，多显示器同步
- **不支持**：输入源切换
- **核心价值**：DDC 通信层的重试和时序控制非常成熟

### 2.2 monitorctl / monitor-switch-ddc（macOS + Windows）

- **仓库**：https://github.com/timurkhakhalev/monitor-switch-ddc
- **技术栈**：Rust，macOS 用 `m1ddc`，Windows 用 Dxva2 API
- **功能**：输入切换 + 托盘预设 + 多显示器配置
- **核心价值**：配置格式、显示器匹配逻辑、预设管理

### 2.3 ddcutil（Linux，1.3K Stars）

- **仓库**：https://github.com/rockowitz/ddcutil
- **技术栈**：C，Linux
- **功能**：全功能 DDC/CI CLI 和共享库
- **核心价值**：重试策略、动态延迟调整、显示器兼容性数据

### 2.4 Rusty Twinkle Tray（Windows）

- **仓库**：https://github.com/sidit77/rusty-twinkle-tray
- **技术栈**：Rust，Windows 原生
- **功能**：亮度控制 + 休眠恢复 + 热插拔
- **核心价值**：亮度恢复机制

### 2.5 ddc-ci-control-bridge（跨平台）

- **仓库**：https://github.com/Defozo/ddc-ci-control-bridge
- **技术栈**：TypeScript/Node.js
- **功能**：全 VCP 读写 + MCP 集成 + MQTT/Home Assistant
- **核心价值**：自动化集成思路（MCP、MQTT）

## 三、借鉴方案详细设计

### 方案 A：DDC 命令重试 + 智能延迟

**来源**：MonitorControl `Arm64DDC.swift` + ddcutil `--maxtries` / `--sleep-multiplier`

**问题**：当前 DDC 操作单次执行，I2C 通信偶发失败时直接报错。

**MonitorControl 的策略**：
- 写操作：每次执行 2 遍（`numOfWriteCycles: 2`），写后等 10ms
- 读操作：写后等 50ms 再读
- 重试：最多 4 次，每次间隔 20ms
- 支持 checksum 校验

**ddcutil 的策略**：
- 写操作默认 4 次重试，读操作默认 10 次重试
- 动态延迟调整：自动跟踪成功率，调节等待时间
- 跨会话记忆每台显示器的最佳参数

**我们的实施方案**：
```rust
// 在 DdcOps trait 实现层加入重试
fn retry_vcp_write(ops: &mut impl DdcOps, code: u8, value: u16) -> Result<(), String> {
    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY_MS: u64 = 50;

    for attempt in 0..=MAX_RETRIES {
        match ops.write_vcp(code, value) {
            Ok(()) => return Ok(()),
            Err(e) if attempt < MAX_RETRIES => {
                let delay = BASE_DELAY_MS * (attempt as u64 + 1); // 50, 100, 150ms
                std::thread::sleep(Duration::from_millis(delay));
                continue;
            }
            Err(e) => return Err(format!("DDC 写入失败（已重试 {} 次）: {}", MAX_RETRIES, e)),
        }
    }
    unreachable!()
}

fn retry_vcp_read(ops: &mut impl DdcOps, code: u8) -> Option<u16> {
    const MAX_RETRIES: u32 = 4;
    const BASE_DELAY_MS: u64 = 40;

    for attempt in 0..=MAX_RETRIES {
        if let Some(val) = ops.read_vcp(code) {
            return Some(val);
        }
        if attempt < MAX_RETRIES {
            std::thread::sleep(Duration::from_millis(BASE_DELAY_MS * (attempt as u64 + 1)));
        }
    }
    None
}
```

**实施位置**：`src-tauri/src/monitor/verify.rs` 或新建 `src-tauri/src/monitor/retry.rs`

**优先级**：P0 | **难度**：低 | **预计版本**：v0.2.0

---

### 方案 B：配置格式 + 显示器匹配

**来源**：monitorctl `config.rs`

**问题**：当前 `AppConfig` 只存输入名称，v0.2.0 需要支持预设和多显示器管理。

**monitorctl 的配置结构**：
```json
{
  "monitors": [
    {
      "match": { "contains": "XG27ACS", "index": null },
      "display": "uuid:XXXX-XXXX",
      "inputs": { "macbook": 15, "pc": 17 }
    }
  ],
  "default_display": "1"
}
```

关键设计点：
- `match.contains`：模糊匹配显示器名称
- `match.index`：精确匹配显示器索引
- `display`：支持 `uuid:XXXX` 或索引号，跨重启稳定
- 每台显示器独立的输入预设

**我们的实施方案**：

扩展 `AppConfig` 结构：
```rust
pub struct AppConfig {
    pub input_names: HashMap<String, String>,     // 现有
    pub monitor_order: Vec<String>,               // 新增：按 model 排序
    pub presets: Vec<InputPreset>,                 // 新增：输入预设
    pub ddc_settings: HashMap<String, DdcCache>,  // 新增：DDC 参数缓存
}

pub struct InputPreset {
    pub name: String,
    pub inputs: HashMap<usize, u16>,  // monitor_index -> input_value
}

pub struct DdcCache {
    pub brightness: Option<u16>,
    pub contrast: Option<u16>,
    pub volume: Option<u16>,
}
```

前端类型对应更新。

**优先级**：P0 | **难度**：中 | **预计版本**：v0.2.0（已在 OpenSpec 规划中）

---

### 方案 C：诊断命令

**来源**：monitorctl `doctor` 命令

**问题**：用户遇到 DDC 不工作时缺少调试手段。

**monitorctl 的做法**：
- 检查 m1ddc 是否安装且路径正确
- 运行 `m1ddc display list` 验证显示器检测
- 输出环境诊断信息

**我们的实施方案**：
- 新增 Tauri 命令 `cmd_diagnose`
- 检查项：m1ddc/ddc-hi 可用性、显示器列表、各 VCP 码支持情况
- 前端在设置页面添加"环境诊断"按钮
- 输出结构化诊断报告，方便用户复制反馈

**优先级**：P1 | **难度**：低 | **预计版本**：v0.2.0

---

### 方案 D：DDC 参数恢复（休眠/唤醒后）

**来源**：Rusty Twinkle Tray

**问题**：用户设置亮度等参数后，显示器休眠/唤醒可能导致设置丢失。

**Rusty Twinkle Tray 的做法**：
- 持久化上次设置的亮度值
- 监听 Windows 电源事件
- 唤醒后延迟一段时间重新写入

**我们的实施方案**：
- 在 `AppConfig` 中保存 `DdcCache`（上次成功设置的值）
- macOS：监听 `NSWorkspace.didWakeNotification`
- Windows/Linux：对应的电源事件
- 唤醒后延迟 2-3 秒重新应用 DDC 设置

**优先级**：P2 | **难度**：中 | **预计版本**：v0.2.1 / v0.3.0

---

### 方案 E：直接 IOKit/IOAVService 访问（替代 m1ddc CLI）

**来源**：MonitorControl `Arm64DDC.swift`

**问题**：通过 `Command::new("m1ddc")` 调用外部进程，有进程启动开销和文本解析不稳定因素。

**MonitorControl 的做法**：
- 直接通过 IOKit 的 `IOAVServiceCreateWithService` / `IOAVServiceWriteI2C` / `IOAVServiceReadI2C` 操作 I2C 总线
- 精确控制写入延迟、读取延迟、重试参数
- 自带 DDC checksum 校验
- 通过 IORegistry 遍历匹配显示器

**我们的实施方案**：
- 使用 Rust 的 `objc2` 或 `core-foundation` crate 调用 IOKit API
- 或将 `m1ddc` 的 Objective-C 核心代码编译为静态库通过 FFI 调用
- 需要新建 `src-tauri/src/monitor/iokit.rs` 模块

**优先级**：P2 | **难度**：高 | **预计版本**：v0.3.0

---

## 四、实施路线图

```
v0.2.0（当前开发中）
├── ✅ 方案 A：DDC 命令重试 + 延迟（P0，低难度）
├── ✅ 方案 B：配置格式扩展（P0，已规划）
└── 可选 方案 C：诊断命令（P1，低难度）

v0.2.1 / v0.3.0
├── 方案 D：DDC 参数恢复（P2，中难度）
└── 方案 E：直接 IOKit 访问（P2，高难度高收益）
```
