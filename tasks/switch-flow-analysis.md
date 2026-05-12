# 输入源切换流程分析与冷却保护

> 创建于 2026-05-07，用于追踪切换逻辑一致性
> 最后更新：2026-07-05

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
| UI 按钮点击 | `MonitorCard.onClick` → `onSwitch(monitor.index, input.value)` → `App.handleSwitch` | ✓ success 5s / warning 8s / error 8s |
| 托盘菜单 | 后端 `tray.rs` → `switch_input()` → emit `tray-switch-done` → 前端设 5s 冷却 + `silentRefresh()` | ✓ |
| 预设应用 | `cmd_apply_preset` 仅后端定义，前端/托盘均无调用入口 | N/A（死代码） |

**统一性保障**：
- 所有输入源按钮（DP-1/DP-2/HDMI-1/HDMI-2/USB-C 等）在 `monitor-card.tsx` L131 统一调用 `onSwitch(monitor.index, input.value)`
- `App.tsx` L426 传入 `onSwitch={handleSwitch}`，无中间代理或条件分支
- `handleSwitch` 内部对 `inputValue` 完全透明，不做任何值特判
- 切换期间 `disabled={isAnySwitching}` 防止并发点击

## 防闪回机制（2026-07-05 重构）

**旧方案（已废弃）**：乐观更新 + `pendingRestore` + `MIN_VISUAL_MS` 延迟恢复
**新方案**：不做乐观更新，切换期间 UI 保持旧状态 + spinner，仅在后端确认后更新

```
handleSwitch:
  T=0        setSwitching(key) → spinner 显示，所有按钮灰化
  T=50ms     invoke 发出（current_input 不变）
  T=~2s      后端返回结果
             success → setMonitors 更新为已确认的 actual_input
             warning → 仅当 actual_input 是已知输入时更新；否则不动
             error   → 不更新 current_input，仅 toast
  T=6s       finally: MIN_VISUAL_MS 等待 → 清除 spinner + switchLock → 显示结果 toast
```

- 无 `pendingRestore`，无 `previousMonitors` 快照
- Warning 路径校验 `actual_input` 是否在 `supported_inputs` 中（过滤 0x00 等无效值）
- Success 路径使用后端已验证的 `actual_input ?? inputValue` 更新 UI
- MIN_VISUAL_MS = 6000ms：确保 spinner 至少显示 6 秒，结果 toast 在 finally 中延迟显示

## 轮询优化（v0.2.6）

三级轮询架构，减少 DDC I/O 和显示器卡顿：

| 级别 | 命令 | 读取内容 | m1ddc 调用数（2台） | 使用场景 |
|------|------|---------|-------------------|---------|
| 完整 | `cmd_get_monitors` | display list + input + brightness/contrast/volume/power | 11 | 首次加载、手动刷新 |
| 轻量枚举 | `cmd_poll_monitors(known=null)` | display list + input | 3 | 首次轮询（未知显示器） |
| 极速轮询 | `cmd_poll_monitors(known=[...])` | 仅 input（跳过 display list） | 2 | 常规定时轮询（5s 间隔） |

前端 `silentRefresh` 合并逻辑：先合并 VCP 值 → 再 JSON 比较，避免因 null VCP 导致每次都触发 state 更新。

## 待验证项

- [x] 用户实际测试：切换到无信号 HDMI-1 后不再闪到"未连接"
- [x] 切换 HDMI-2 不再显示 "Input-0x00" — 前端校验 actual_input 有效性，无效值不更新 UI
- [x] 所有输入源按钮走同一条 handleSwitch 路径
- [ ] 预设应用（cmd_apply_preset）路径是否也有类似保护（当前为死代码，无前端调用入口）
- [ ] 多显示器场景：切换其中一台时另一台不受影响

## 验证逻辑（verify_switch）

验证共 3 轮（600ms、1400ms、2000ms 延迟后读取），处理 4 种读取结果：

| 读取结果 | 处理 |
|---------|------|
| actual == target（归一化后） | confirmed = true，重置不可达计数 |
| actual 是已知输入 ≠ target | 返回 warning "无信号，已恢复到 {actual}" |
| actual 是无效值（如 0x00） | 视为过渡态，跳过本轮，重置不可达计数（DDC 有响应） |
| None（DDC 不可达） | 累加连续不可达计数，连续 2 次则提前终止验证 |

最终判定：
- confirmed = true → success
- confirmed = false → 尝试回滚到 previous_input

## 时间线参考

```
T=0       handleSwitch → setSwitching(key) 阻塞 UI
T=0.05    后端 invoke 发出
T=0.6-4s  后端 verify_switch（3轮：600ms + 1400ms + 2000ms，可能提前终止）
T=~4s     invoke 返回 → 设置 revertCooldownUntil
T=6s      finally → MIN_VISUAL_MS 等待 → switchingRef 清除 → 显示结果 toast
T=6-11s   冷却保护期（success 5s / warning|error 8s）：空结果被忽略
T=11-14s+ 保护解除，正常轮询恢复（5s 间隔，仅读 input）
```

## 测试覆盖

### 后端 verify_switch（16 tests in `src-tauri/src/monitor/verify.rs`）

| 场景分类 | 测试名 | 验证点 |
|---------|--------|--------|
| 成功 | `verify_success_both_rounds_confirm` | 两轮都读到目标值 → success |
| 成功 | `verify_success_first_round_confirms_second_jitters` | 第一轮确认后第二轮 DDC 抖动 → 仍 success |
| 成功 | `verify_success_vendor_code_equivalence` | 0x6E == 0x0F（归一化） |
| 成功 | `verify_target_is_vendor_code_matches_standard` | 目标 0x6E，读回 0x0F → success |
| 成功 | `verify_no_writes_on_successful_switch` | 成功时不触发任何写操作 |
| 失败 | `verify_warning_different_known_input` | 读回不同已知输入 → warning "无信号" |
| 无效值 | `verify_invalid_value_treated_as_transient` | 两轮都 0x00 → 不判为已知输入回弹 |
| 无效值 | `verify_invalid_then_target_confirms` | 第一轮 0x00 跳过，第二轮确认 → success |
| 无效值 | `verify_multiple_invalid_values_all_skipped` | 不同无效值（0x63, 0xFE）都跳过 |
| None | `verify_none_round0_continues_to_next_round` | Round 0 不可达 → 跳过，继续下一轮 |
| None | `verify_none_round0_no_previous_errors` | 全部 None + 无前输入时回滚 → error |
| None | `verify_none_later_rounds_not_confirmed` | 已确认后 None → 仍 success |
| None | `verify_all_none_after_round0_not_confirmed` | 无效值 + None → "可能无信号" |
| 回滚 | `rollback_write_fails_returns_error` | 回滚写入失败 → error |
| 回滚 | `verify_none_then_different_known_input` | None 后读到已知输入 → warning |
| 回滚 | `rollback_write_ok_but_readback_none` | 回滚后仍不可达 → "无法确认" |

### 前端切换冷却保护（4 tests in `src/__tests__/app.test.tsx`）

| 测试名 | 验证点 |
|--------|--------|
| `preserves monitors when post-switch poll returns empty during cooldown` | success 路径：5s 冷却内轮询返回空不清 |
| `preserves monitors when display-changed triggers refresh with empty result during cooldown` | warning 路径：8s 冷却 + display-changed 事件 |
| `preserves monitors when switch invoke rejects (error path)` | error 路径：8s 冷却保护 |
| `tray-switch-done event sets cooldown and preserves monitors` | 托盘路径：5s 冷却 + display-changed |

### 测试运行方式

```bash
# 后端
cd src-tauri && cargo test

# 前端
npx vitest run

# 仅切换相关
cd src-tauri && cargo test verify
npx vitest run -t "cooldown"
```
