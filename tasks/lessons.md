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
setMonitors(optimistic update);
showToast("正在切换输入源...");

// 2. 让出事件循环，确保浏览器渲染
await new Promise(r => setTimeout(r, 50));

// 3. 执行后端操作
const result = await invoke("cmd_switch_input", ...);

// 4. finally 中确保最小显示时间
const elapsed = Date.now() - switchStart;
if (elapsed < 600) await new Promise(r => setTimeout(r, 600 - elapsed));
setSwitching(null);
```

按钮禁用样式（内联样式，绕过所有 CSS 干扰）：
```tsx
variant={isThisSwitching || (!isAnySwitching && isActive) ? "default" : "outline"}
style={isAnySwitching && !isThisSwitching
  ? { opacity: 0.35, pointerEvents: "none", filter: "grayscale(100%)",
      transition: "none", background: "transparent", color: "inherit" }
  : undefined}
```

### 核心教训总结

1. **简单方案优先**：`setTimeout(50)` 比 `requestAnimationFrame`、`flushSync` 都更可靠。不要过度工程化
2. **不要假设浏览器 API 行为一致**：WKWebView ≠ Chrome，rAF 和 flushSync 行为不同
3. **CSS 类不可靠时用内联样式**：第三方组件 + twMerge + CSS 伪类 = 不可预测，内联样式最高优先级
4. **每次修改后必须实际安装测试**：开发模式 ≠ 生产构建，DMG 安装可能未替换旧版本
5. **一次只改一件事**：多个问题叠加时，逐个修复和验证，不要同时改多个
6. **记录失败尝试**：避免团队成员重复踩坑

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
