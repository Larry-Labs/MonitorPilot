# 经验教训

## 2026-04-14: 切换状态视觉反馈问题（多次迭代教训）

### 问题现象
切换输入源时，其他按钮没有显示禁用/灰化效果，底部 "正在切换输入源..." 提示也不出现或不稳定。

### 根因分析（4 个叠加问题）

**1. @base-ui/react Button 的 disabled 行为与 CSS 不匹配**
- `@base-ui/react` Button 使用 `data-disabled` 属性，而非原生 HTML `disabled`
- shadcn/ui buttonVariants 的 `disabled:opacity-50` 依赖 CSS `:disabled` 伪类 → 完全失效
- **解决**：使用内联 `style={{ opacity, pointerEvents, filter }}` 直接控制，绕过 CSS 类和 twMerge
- **教训**：第三方 headless UI 库的 disabled 行为不可假设，必须验证实际 DOM 输出

**2. React 18 批处理 + 快速 async 操作导致中间状态不可见**
- `setSwitching` + `showToast("switching")` 后立即 `await invoke(...)`
- 如果 invoke 解析极快，React 将 switching → success 状态合并渲染，用户看不到中间态
- **解决**：在 invoke 前插入 `await new Promise(r => setTimeout(r, 50))` 让出事件循环
- **教训**：需要用户看到中间状态时，必须在 async 操作前通过 setTimeout 强制渲染

**3. DDC 操作可能极快完成（< 200ms）**
- **解决**：`MIN_VISUAL_MS = 600` 确保切换状态至少保持 600ms
- **教训**：涉及用户感知的状态变化应设置最小显示持续时间

**4. 之前活跃按钮从 variant="default"（蓝色）到 "outline" 的过渡导致视觉不一致**
- 切换后原按钮 variant 变化 + grayscale 会比其他 outline 按钮看起来更深
- **解决**：切换期间强制所有非目标按钮使用 `variant="outline"`，并用内联样式 `background: transparent` 覆盖

### 错误尝试记录（重要！避免重复）

| 尝试 | 方案 | 结果 | 问题 |
|------|------|------|------|
| 1 | CSS 类 `opacity-40 cursor-not-allowed` | 无效果 | base-ui 不设置原生 disabled，CSS :disabled 伪类不匹配 |
| 2 | `requestAnimationFrame` 双帧等待 | 开发模式正常，生产不稳定 | WKWebView 中 rAF 回调不可靠 |
| 3 | `flushSync()` 强制同步渲染 | 完全回退，啥都没了 | flushSync 更新 DOM 但不触发浏览器绘制 |
| 4 | `flushSync()` + `setTimeout(16)` | 同上 | flushSync 在 Tauri WKWebView 中有兼容性问题 |
| 5 | **`setTimeout(50)`** | **稳定正常** | 简单可靠，跨环境兼容 |

### 最终修复方案

```typescript
// 1. 设置状态（React 批处理）
setSwitching(key);
showToast("正在切换输入源...");

// 2. 让出事件循环，确保浏览器渲染
await new Promise(r => setTimeout(r, 50));

// 3. 执行后端操作
const result = await invoke("cmd_switch_input", ...);

// 4. finally 中确保最小显示时间（MIN_VISUAL_MS = 6000ms）
const elapsed = Date.now() - switchStart;
if (elapsed < MIN_VISUAL_MS) await new Promise(r => setTimeout(r, MIN_VISUAL_MS - elapsed));
setSwitching(null);
// 结果 toast 在 finally 中延迟显示，与 spinner 清除同步
```

按钮切换样式（纯 Tailwind 类，不使用内联 style）：
```tsx
variant={!isAnySwitching && isActive ? "default" : "outline"}
className={`... ${
  isThisSwitching
    ? "!border-primary !text-primary shadow-md pointer-events-none transition-none"
    : isAnySwitching
      ? "opacity-35 grayscale pointer-events-none transition-none"
      : isActive
        ? "shadow-md shadow-primary/20"
        : "hover:border-primary/40 ..."
}`}
disabled={isAnySwitching && !isThisSwitching}
```

### 核心教训总结

1. **简单方案优先**：`setTimeout(50)` 比 `requestAnimationFrame`、`flushSync` 都更可靠。不要过度工程化
2. **不要假设浏览器 API 行为一致**：WKWebView ≠ Chrome，rAF 和 flushSync 行为不同
3. **CSS 变量格式必须匹配**：`--primary` 用 oklch 格式时，`hsl(var(--primary))` 是无效 CSS。直接用 `var(--primary)` 或 Tailwind 类 `border-primary`
4. **Tailwind 类优先于内联样式**：特别是 Tailwind v4，内联 style 可能被 CSS 层级覆盖。用 `!border-primary`（v4 前缀 `!` 语法）+ tailwind-merge 覆盖
5. **CVA 基础类不可移除**：`buttonVariants` 基础类中的 `transition-all` 无法通过 className 删除（除非 tailwind-merge 正确处理）。需要用 `transition-none` 显式覆盖
6. **ternary 判断顺序很关键**：`isAnySwitching` 必须在 `isActive` 前判断，否则当前激活按钮不会进入灰色状态
7. **每次修改后必须实际安装测试**：开发模式 ≠ 生产构建，DMG 安装可能未替换旧版本
8. **一次只改一件事**：多个问题叠加时，逐个修复和验证，不要同时改多个
9. **记录失败尝试**：避免团队成员重复踩坑
10. **JSON 比较必须在合并后进行**：轮询返回 null 的字段合并旧值后，用合并结果比较，否则每次都触发无意义的 state 更新
11. **DDC 轮询要分级**：完整读取（input + 4 个 VCP）会造成显示器卡顿。常规轮询只读 input，VCP 按需加载

## 2026-07-05: DDC 轮询导致显示器卡顿

### 问题
用户报告 UI 和鼠标周期性卡顿。

### 根因
`cmd_get_monitors` 每 3 秒读取 1 次 display list + 每台显示器 5 个 DDC 值（input + brightness + contrast + volume + power），2 台显示器 = 11 次 m1ddc 进程启动 / 11 次 DDC I2C 操作。DDC/CI 通信会导致显示器固件短暂冻结显示输出。

### 解决
三级轮询：
- `get_monitors()`：完整读取（首次加载）
- `get_monitors_light()`：仅 display list + input（首次轮询）
- `poll_inputs(known)`：跳过 display list，仅读已知显示器的 input（常规轮询）

效果：2 台显示器从 11 次 → 2 次 m1ddc 调用，间隔从 3s → 5s。

### 教训
- DDC/CI I2C 操作是显示器卡顿的主因，减少操作次数是最有效优化
- 不需要每次轮询都重新枚举显示器列表（display-changed 事件已覆盖）
- VCP 值（亮度/对比度/音量/电源）只在 DDC 控制面板打开时才需要读取

## 2026-04-14: 切换失败后 UI 恢复到错误端口

### 问题
从 HDMI-1 切到 HDMI-2 失败后，显示器自动跳到 DP-1，但 UI 显示 HDMI-1（使用的是 previousMonitors 快照）。

### 解决
后端 `SwitchResult` 新增 `actual_input` 字段，返回显示器实际所在端口。前端据此更新 UI 而非盲目恢复快照。

### 教训
- 显示器的自动输入检测可能跳到任意有信号的端口，不一定是原来的
- 前端恢复逻辑不能假设"失败 = 回到原来的端口"

## 2026-04-14: 深度 Review 发现的系统性问题

### @base-ui/react 组件库迁移遗留

shadcn/ui 最新版从 Radix 迁移到 @base-ui/react，`disabled` 行为从原生 HTML 属性变为 `data-disabled`。所有 UI 基础组件中的 `disabled:` CSS 伪类需替换为 `data-[disabled]:`。受影响组件：Button、Slider Thumb、Input。

**教训**：升级 UI 库底层实现后，必须逐一验证每个基础组件的 DOM 输出，不能只看 API 是否兼容。

### verify_switch 验证逻辑不严谨

验证循环中，若后续轮次 `read_input()` 返回 `None`（DDC 暂时不可达），循环结束后仍返回 `success`。实际上最后一次读取并未确认输入源仍为目标值。

**修复**：引入 `confirmed` 标志，仅在最后一轮确认成功时才返回 success；否则返回 warning。

### React Strict Mode 下 setMonitors 回调双调用

`setMonitors(prev => { previousMonitors = prev; ... })` 在 Strict Mode 下回调被调两次，`previousMonitors` 可能被覆盖为乐观更新后的值。

**修复**：用 `monitorsSnapshotRef` 始终保持最新的 monitors 镜像，不依赖 updater 回调捕获快照。

### DdcControls 静默吞错

DDC 滑块和电源按钮的 `invoke` 失败仅 `console.error`，用户完全无感知。

**修复**：添加 `onError` 回调，由父组件通过 toast 展示。

### [已知限制] desktop.rs 使用枚举顺序作为显示器 index

Linux/Windows 路径中 `desktop.rs` 用 `Display::enumerate()` 的迭代顺序作为 `monitor.index`。`switch_input` 和 `set_vcp` 每次重新 `enumerate()` 并用 `nth(index)` 取显示器。如果两次枚举之间发生热插拔、睡眠/唤醒等，同一个 index 可能指向不同物理显示器，导致**写错屏**。

macOS 路径使用 m1ddc 返回的 `display_num`（相对稳定），此问题主要影响 Linux/Windows。

**未来改进方向**：使用 EDID、设备路径或序列号作为稳定标识；或在单次持锁流程内缓存枚举结果。

## 2026-07-05: 切换输入源 UI 闪回 Bug（前后端联合问题）

### 问题现象

从 DP-1 切换到未连接的端口（如 DP-2/HDMI-2）时，UI 上先显示目标输入为"当前"（蓝色高亮），然后约 1.5 秒后闪回到原来的 DP-1。物理显示器也可能短暂黑屏后恢复。

用户描述："DP-1切DP-2，光标卡住了一会，然后移动到了DP-2上，然后瞬间切换回DP-1"

### 根因分析

**两层问题叠加：**

#### 根因 1：前端乐观更新（UI 闪回主因）

`handleSwitch` 在调用后端 `cmd_switch_input` **之前**就通过 `setMonitors()` 将 `current_input` 设为目标值（乐观更新）。当切换失败时，`pendingRestore` 将状态恢复回原值，造成 UI 上可见的"先跳到目标 → 1.5 秒后闪回"。

```typescript
// 问题代码（已移除）
setMonitors(prev => prev.map(m => ({
  ...m, current_input: inputValue,  // ← 未确认就修改
  current_input_name: targetInput?.name ?? m.current_input_name,
})));
```

#### 根因 2：后端验证过早回滚（物理闪回加剧因）

`verify_switch` 验证第 1 轮（600ms 后）`read_input()` 返回 `None`（DDC 暂时不可达）时，立即调用 `attempt_rollback` 发送 DDC write 切回原输入。但 600ms 对某些显示器不够完成物理切换，DDC 只是暂时不可达。回滚命令导致显示器在切换完成后又被强制切回。

```rust
// 问题代码（已修复）
None if round == 0 => {
    return attempt_rollback(target_value, previous_input, ops);
}
```

#### 附加问题：后端返回无效 actual_input

后端 `attempt_rollback` 可能返回 `actual_input: 0x00`（DDC 垃圾值），前端盲目将 `current_input` 设为 0x00，导致没有任何按钮显示为"当前"。

### 修复方案

#### 前端修复（App.tsx handleSwitch）

1. **移除乐观更新**：不再在调用后端前修改 `current_input`，切换期间 UI 保持旧状态 + spinner
2. **成功时确认更新**：后端验证通过后，用 `result.actual_input`（已确认值）更新 UI
3. **Warning 路径校验 actual_input**：仅当 `actual_input` 匹配已知 `supported_inputs` 时才更新 UI，过滤掉 0x00 等无效值
4. **移除 pendingRestore 模式**：不再需要延迟恢复（因为没有需要恢复的乐观更新）
5. **移除 previousMonitors 快照**：不再需要旧状态备份

#### 后端修复（verify.rs verify_switch）

1. **Round 0 None 不再触发立即回滚**：改为跳过，继续等待下一轮验证
2. **给显示器更多时间**：从 600ms 增加到完整 2000ms（两轮完毕）后才考虑回滚
3. **统一 None 处理**：所有轮次的 None 行为一致——记录日志并跳过

### 修复后的行为对比

| 场景 | 修复前 | 修复后 |
|------|--------|--------|
| 切到未连接端口 | UI 先显示目标 → 1.5s 后闪回 + warning | UI 始终保持原输入 + spinner → warning |
| 切换成功 | UI 立即显示目标（乐观） | UI 保持原输入 + spinner → 后端确认后更新 |
| 切换报错 | UI 先显示目标 → 闪回 + error | UI 保持原输入 + spinner → error |
| DDC 读回 0x00 | UI current_input 设为 0x00（无按钮高亮） | 过滤无效值，UI 不变 |

### 教训

1. **乐观更新需要高成功率前提**：DDC/CI 切换成功率不可预测（取决于硬件、信号），不适合乐观更新
2. **后端不应在不确定时主动回滚**：DDC 不可达可能是暂时的（切换中），过早回滚反而造成物理闪回
3. **前端要校验后端返回的值**：DDC 读取结果可能是垃圾值（0x00），不能无条件用于 UI 更新
4. **物理设备操作的验证需要足够的等待时间**：显示器物理切换 + DDC 恢复可能需要 2-3 秒，600ms 太短
5. **UI 状态变更应基于确认而非预测**：只有后端明确确认的状态才能用于更新 UI
