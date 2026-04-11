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

## 12. 构建验证

- [x] 12.1 运行 `cargo check` 确认后端无警告
- [x] 12.2 运行 `npm run tauri build` 确认构建成功
