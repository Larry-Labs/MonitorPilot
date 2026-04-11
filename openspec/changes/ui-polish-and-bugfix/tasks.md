## 1. 后端 Bug 修复

- [x] 1.1 修复 `tray.rs` 中 TrayIconBuilder 未设置 id，确保 `refresh_tray` 能通过 `tray_by_id("main")` 找到托盘实例
- [x] 1.2 移除 `config.rs` 中未使用的 `update_input_name` 方法
- [x] 1.3 修复 `tray.rs` 中 `handle_menu_event` 切换失败被静默吞掉的问题，添加日志输出

## 2. 前端错误处理

- [x] 2.1 为 `App.tsx` 中 `cmd_get_config` 调用添加 `.catch` 错误处理
- [x] 2.2 修复 `monitor-card.tsx` 中 `onSwitch` 第三参数未使用的问题，对齐接口定义

## 3. 类型重构

- [x] 3.1 创建 `src/types/monitor.ts`，提取 `InputSource` 和 `MonitorInfo` 类型定义
- [x] 3.2 更新 `App.tsx` 和 `monitor-card.tsx` 从共享类型文件导入

## 4. 暗色模式

- [x] 4.1 在 `index.html` 中添加内联脚本，根据 `prefers-color-scheme` 在页面渲染前设置 `dark` 类
- [x] 4.2 修复 `index.html` 的 `lang` 属性为 `zh-CN`

## 5. UI 美化

- [x] 5.1 添加 shadcn/ui Skeleton 组件，实现加载骨架屏
- [x] 5.2 使用 shadcn/ui Alert 组件替代当前的纯文本错误提示
- [x] 5.3 重做空状态：使用图标 + 描述 + 操作建议的结构化布局
- [x] 5.4 为输入源添加显式编辑按钮图标，替代仅双击触发重命名
- [x] 5.5 将输入源网格从固定 `grid-cols-3` 改为响应式布局
- [x] 5.6 主题色从纯灰替换为蓝紫色品牌色方案
- [x] 5.7 功能提示面板移除（用户反馈不需要）

## 6. 项目合规

- [x] 6.1 在根目录创建 MIT LICENSE 文件
- [x] 6.2 确认 `package-lock.json` 已纳入 Git 跟踪

## 7. m1ddc 打包

- [x] 7.1 将 m1ddc 二进制通过 Tauri externalBin 打包进 app
- [x] 7.2 运行时优先使用 sidecar 路径，fallback 到系统 PATH

## 8. 错误处理优化

- [x] 8.1 修复 m1ddc 错误信息在 stdout 而非 stderr 的问题
- [x] 8.2 过滤内置显示器（不支持 DDC/CI）
- [x] 8.3 添加 m1ddc display list 退出码检查
- [x] 8.4 错误和空状态添加"重新检测"按钮
- [x] 8.5 切换操作全局提示条（正在切换/成功/失败恢复）

## 9. 托盘菜单完善

- [x] 9.1 顶部添加 "MonitorPilot vX.X.X" 标题
- [x] 9.2 托盘菜单中使用自定义名称（从 config 读取）
- [x] 9.3 添加 "帮助" 子菜单（关于/项目主页/反馈问题）
- [x] 9.4 去掉 emoji，纯文字更正式
- [x] 9.5 版本号通过 CARGO_PKG_VERSION 自动获取
- [x] 9.6 "访问项目主页"/"反馈问题" 点击打开浏览器

## 10. Code Review 修复（3轮）

- [x] 10.1 小窗口滚动条：h-screen + overflow-hidden
- [x] 10.2 删除未使用 shadcn 组件（scroll-area/tooltip/separator）
- [x] 10.3 删除未使用 npm 依赖（plugin-fs/plugin-shell/lucide-react）
- [x] 10.4 tray.rs unwrap → 安全错误处理 + refresh_tray 日志
- [x] 10.5 handleRename 保存失败回滚状态
- [x] 10.6 Footer 版本号通过 vite define 自动注入
- [x] 10.7 macOS 从 m1ddc 行内解析 [N] 作为显示器编号
- [x] 10.8 消除 parse_m1ddc_display_name 死分支
- [x] 10.9 macos_get_input 检查退出码
- [x] 10.10 Release 构建启用日志插件
- [x] 10.11 Footer 对比度提升

## 11. 热插拔与状态显示

- [x] 11.1 实现 5s 定时轮询，自动检测显示器连接/断开变化
- [x] 11.2 窗口不可见时暂停轮询，可见时恢复
- [x] 11.3 切换操作期间暂停轮询避免冲突
- [x] 11.4 显示器卡片副标题显示"当前输入"信息

## 12. 输入状态显示增强

- [x] 12.1 活跃输入按钮显示绿色脉冲圆点 + "当前"标签
- [x] 12.2 Header Badge 同步显示当前输入名称
- [x] 12.3 动态输入源列表：自动添加当前输入值到支持列表（即使不在预定义列表中）

## 13. 并发切换保护

- [x] 13.1 使用 useRef 互斥锁防止快速连点并发切换
- [x] 13.2 切换完成后使用 silentRefresh 替代 refreshMonitors，避免 loading 骨架屏

## 14. 切换失败自动回滚

- [x] 14.1 switch_input 切换前记录当前输入值
- [x] 14.2 切换后 monitor 不可达时（get_input 返回 None），自动发送回滚命令恢复原输入
- [x] 14.3 回滚成功/失败均有明确错误消息返回前端
- [x] 14.4 前端轮询保护：silentRefresh 在已有显示器状态下不会因瞬态空结果清空 UI

## 15. 全流程日志

- [x] 15.1 monitor.rs 检测流程日志（m1ddc 路径、输出、跳过内置显示器、检测结果）
- [x] 15.2 monitor.rs 切换流程日志（切换请求、命令结果、验证结果、回滚过程）
- [x] 15.3 日志级别：debug 用于详细输出，info 用于关键事件，warn/error 用于异常

## 16. 代码审查修复（第4轮）

- [x] 16.1 lib.rs: `cmd_switch_input` 无论成功失败都刷新托盘（修复"托盘刷新不及时"）
- [x] 16.2 lib.rs: 移除未使用的 `tauri_plugin_fs::init()`
- [x] 16.3 Cargo.toml: 移除 `tauri-plugin-fs` 依赖
- [x] 16.4 capabilities: 移除未使用的 `fs:default` 权限
- [x] 16.5 App.tsx: toast timer 卸载时清理，防止内存泄漏
- [x] 16.6 App.tsx: 轮询 effect 使用 `switchingRef` 避免 `switching` 变化重建 interval
- [x] 16.7 App.tsx: `silentRefresh` catch 添加日志输出
- [x] 16.8 App.tsx: Toast 容器添加 `role="status"` + `aria-live="polite"`
- [x] 16.9 monitor.rs: 移除多余的 `use log;` 导入
- [x] 16.10 config.rs: Mutex `lock().unwrap()` → `unwrap_or_else` 恢复被污染的锁
- [x] 16.11 config.rs: 配置文件解析失败时记录 `log::warn`
- [x] 16.12 tray.rs: `open::that` 错误记录日志
- [x] 16.13 tray.rs: 菜单 ID 解析失败记录日志

## 17. 构建验证

- [x] 17.1 运行 `cargo check` 确认后端无警告
- [x] 17.2 运行 `npm run tauri build` 确认构建成功
- [x] 17.3 TypeScript 编译检查通过

## 18. 代码审查修复（10 轮深度审查）

- [x] 18.1 lib.rs: 添加 DDC_LOCK 互斥锁，保证后端 DDC 操作串行化
- [x] 18.2 tauri.conf.json: 设置 CSP 安全策略（替代 `csp: null`）
- [x] 18.3 Cargo.toml: `ddc-hi` 改为条件依赖，仅 Linux/Windows 编译
- [x] 18.4 monitor.rs: `find_m1ddc()` 使用 OnceLock 缓存路径，避免重复查找
- [x] 18.5 monitor.rs: `parse_m1ddc_line` 解析失败时 log::warn 而非静默默认值
- [x] 18.6 App.tsx: `TOAST_COLORS` 常量移至组件外部，避免每次渲染重建
- [x] 18.7 App.tsx: `handleSwitch` / `handleRename` 包装为 `useCallback`，减少子组件重渲染
- [x] 18.8 App.tsx: 使用 `customNamesRef` 避免 `handleRename` 对 `customNames` 状态的闭包依赖
- [x] 18.9 monitor-card.tsx: 使用 `React.memo` 包装组件，配合 `useCallback` 优化性能
- [x] 18.10 monitor-card.tsx: 添加 `aria-pressed` 和 `aria-label` 提升可访问性
- [x] 18.11 monitor-card.tsx: 编辑按钮从 `hidden` 改为 `opacity` 方案，保持 Tab 可聚焦
- [x] 18.12 monitor-card.tsx: 长名称添加 `truncate` + `title` 属性防止溢出
- [x] 18.13 tray.rs: 帮助子菜单 About 添加 "by Larry Gao" 作者信息

## 19. 托盘菜单扁平化

- [x] 19.1 去除"帮助"子菜单，"访问项目主页"/"反馈问题"提到主菜单
- [x] 19.2 顶部标题合并为 "MonitorPilot vX.X.X — Larry Gao"，正常颜色显示
- [x] 19.3 去除版本号和作者信息的重复项

## 20. 代码审查修复（5 轮深度审查）

- [x] 20.1 DDC_LOCK 下沉到 monitor.rs switch_input 内部，保护所有调用路径（含托盘）
- [x] 20.2 移除未使用的 tauri-plugin-shell 依赖和 shell:default 权限
- [x] 20.3 lib.rs .expect() 改为 unwrap_or_else 安全退出
- [x] 20.4 所有 `let _ =` 静默丢弃改为 log::warn 记录
- [x] 20.5 cmd_save_config 失败时记录 log::error
- [x] 20.6 Cargo.toml repository 填充实际仓库 URL
- [x] 20.7 Cargo.toml authors 统一为 "Larry Gao"
- [x] 20.8 OpenSpec spec 文档同步：托盘扁平化、DDC_LOCK 下沉、作者位置
