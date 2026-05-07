# 输入源切换流程分析与冷却保护

> 创建于 2026-05-07，用于追踪切换逻辑一致性

## 问题背景

切换到无信号端口（如 HDMI-1 物理未连接）后，UI 闪到"未连接屏幕"状态。

## 根因

`refreshMonitors()`（由 `display-changed` 事件触发）**没有空结果保护**，直接 `setMonitors([])`。
切换后 macOS CoreGraphics 会触发 display reconfiguration 事件，如果此时 DDC 暂时不可达，
`get_monitors` 返回空 → UI 立即清空。

## 修复方案

### 冷却期保护（revertCooldownUntil）

所有切换路径都设置冷却期：

| 路径 | 冷却时长 | 说明 |
|------|---------|------|
| success | 5 秒 | 切换成功但显示器可能短暂 DDC 中断 |
| warning | 8 秒 | 切换失败/回滚，显示器恢复需更长时间 |
| error (catch) | 8 秒 | 后端异常，保守等待 |

### 保护检查点

两个入口都检查冷却期：

1. **`refreshMonitors()`**（由 display-changed 事件触发）：
   ```ts
   if (result.monitors.length === 0 && Date.now() < revertCooldownUntil.current) {
     return; // 不清空
   }
   ```

2. **`silentRefresh()`**（由定时轮询触发）：
   ```ts
   if (Date.now() < revertCooldownUntil.current) return;
   ```

### 阻塞检查点（切换期间）

3. **`switchingRef.current`**（切换进行中阻塞事件）：
   - `display-changed` handler: `if (!switchingRef.current) refreshMonitors()`
   - `silentRefresh`: `if (!switchingRef.current) silentRefresh()` in setInterval
   - 生命周期：handleSwitch 开始时设置 → finally 块中清除

## 切换入口清单

确认所有触发切换的入口都走统一的 `handleSwitch` 逻辑：

| 入口 | 调用路径 | 冷却保护 |
|------|---------|---------|
| UI 按钮点击 | `MonitorCard` → `handleSwitch()` | ✓ success 5s / warning 8s / error 8s |
| 托盘菜单 | 后端 `tray.rs` → `switch_input()` → emit `tray-switch-done` → 前端设 5s 冷却 + `silentRefresh()` | ✓ |
| 预设应用 | `cmd_apply_preset` 仅后端定义，前端/托盘均无调用入口 | N/A（死代码） |

## 待验证项

- [x] 用户实际测试：切换到无信号 HDMI-1 后不再闪到"未连接"
- [x] 切换 HDMI-2 不再显示 "Input-0x00" — 改为显示 "HDMI-2 可能无信号，已恢复到 DP-1"
- [x] 所有输入源按钮走同一条 handleSwitch 路径
- [ ] 预设应用（cmd_apply_preset）路径是否也有类似保护（当前为死代码，无前端调用入口）
- [ ] 多显示器场景：切换其中一台时另一台不受影响

## 验证逻辑（verify_switch）

验证共 2 轮（600ms + 1400ms 延迟后读取），处理 4 种读取结果：

| 读取结果 | 处理 |
|---------|------|
| actual == target（归一化后） | confirmed = true |
| actual 是已知输入 ≠ target | 返回 warning "无信号，已恢复到 {actual}" |
| actual 是无效值（如 0x00） | 视为过渡态，跳过本轮（不判定失败） |
| None（DDC 不可达） | Round 0: attempt_rollback; 其他: 跳过 |

最终判定：
- confirmed = true → success
- confirmed = false → warning "可能无信号，已恢复到 {previous_input}"

## 时间线参考

```
T=0       handleSwitch → switchingRef 阻塞
T=0.05    后端 invoke 发出
T=0.6-3s  后端 verify_switch（600ms + 1400ms）
T=~3s     invoke 返回 → 设置 revertCooldownUntil
T=~3.6s   finally → switchingRef 清除
T=3.6-11s 冷却保护期：空结果被忽略
T=11s+    保护解除，正常轮询恢复（此时显示器已稳定）
```
