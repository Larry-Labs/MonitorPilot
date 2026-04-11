use std::sync::Arc;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::config::ConfigManager;
use crate::monitor::{get_monitors, switch_input};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("默认窗口图标未设置")?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("MonitorPilot")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            handle_menu_event(app, id);
        })
        .build(app)?;

    log::info!("系统托盘初始化完成");
    Ok(())
}

fn build_tray_menu(
    app: &AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mut builder = MenuBuilder::new(app);

    let title = MenuItemBuilder::with_id(
        "title",
        format!("MonitorPilot v{} — Larry Gao", APP_VERSION),
    )
    .build(app)?;
    builder = builder.item(&title);

    builder = builder.separator();

    let custom_names = app
        .try_state::<Arc<ConfigManager>>()
        .map(|cm| cm.get().input_names)
        .unwrap_or_default();

    match get_monitors() {
        Ok(monitors) if !monitors.is_empty() => {
            for monitor in &monitors {
                let submenu = build_monitor_submenu(app, monitor, &custom_names)?;
                builder = builder.item(&submenu);
            }
        }
        Ok(_) => {
            let item = MenuItemBuilder::with_id("no_monitors", "未检测到 DDC/CI 显示器")
                .enabled(false)
                .build(app)?;
            builder = builder.item(&item);
        }
        Err(e) => {
            let item = MenuItemBuilder::with_id("error", format!("检测异常: {}", e))
                .enabled(false)
                .build(app)?;
            builder = builder.item(&item);
        }
    }

    builder = builder.separator();

    let refresh = MenuItemBuilder::with_id("refresh", "刷新显示器列表").build(app)?;
    builder = builder.item(&refresh);

    let settings = MenuItemBuilder::with_id("settings", "打开主界面...").build(app)?;
    builder = builder.item(&settings);

    builder = builder.separator();

    let homepage = MenuItemBuilder::with_id("homepage", "访问项目主页").build(app)?;
    builder = builder.item(&homepage);

    let report_issue = MenuItemBuilder::with_id("report_issue", "反馈问题").build(app)?;
    builder = builder.item(&report_issue);

    builder = builder.separator();

    let quit = MenuItemBuilder::with_id("quit", "退出 MonitorPilot").build(app)?;
    builder = builder.item(&quit);

    Ok(builder.build()?)
}

fn build_monitor_submenu(
    app: &AppHandle,
    monitor: &crate::monitor::MonitorInfo,
    custom_names: &std::collections::HashMap<String, String>,
) -> Result<tauri::menu::Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mut submenu = SubmenuBuilder::new(app, &monitor.model);

    for input in &monitor.supported_inputs {
        let is_active = monitor.current_input == Some(input.value);
        let item_id = format!("switch_{}_{}", monitor.index, input.value);
        let key = format!("{}-{}", monitor.index, input.value);
        let display_name = custom_names.get(&key).unwrap_or(&input.name);
        let label = if is_active {
            format!("✓ {}", display_name)
        } else {
            format!("   {}", display_name)
        };

        let item = MenuItemBuilder::with_id(item_id, label)
            .enabled(!is_active)
            .build(app)?;
        submenu = submenu.item(&item);
    }

    Ok(submenu.build()?)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "quit" => {
            app.exit(0);
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    log::warn!("托盘: 显示窗口失败: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    log::warn!("托盘: 聚焦窗口失败: {}", e);
                }
            }
        }
        "refresh" => {
            refresh_tray(app);
        }
        "homepage" => {
            if let Err(e) = open::that("https://github.com/Larry-Labs/MonitorPilot") {
                log::error!("打开项目主页失败: {}", e);
            }
        }
        "report_issue" => {
            if let Err(e) = open::that("https://github.com/Larry-Labs/MonitorPilot/issues") {
                log::error!("打开反馈页面失败: {}", e);
            }
        }
        id if id.starts_with("switch_") => {
            let parts: Vec<&str> = id.split('_').collect();
            if parts.len() == 3 {
                if let (Ok(monitor_idx), Ok(input_val)) =
                    (parts[1].parse::<usize>(), parts[2].parse::<u8>())
                {
                    match switch_input(monitor_idx, input_val) {
                        Ok(result) if result.status == "warning" => {
                            log::warn!("托盘切换警告: {}", result.message);
                        }
                        Ok(result) => {
                            log::info!("托盘切换成功: {}", result.message);
                        }
                        Err(e) => log::error!("托盘切换失败: {}", e),
                    }
                    refresh_tray(app);
                } else {
                    log::warn!("托盘菜单 ID 解析失败: {}", id);
                }
            } else {
                log::warn!("托盘菜单 ID 格式异常: {}", id);
            }
        }
        _ => {}
    }
}

pub fn refresh_tray(app: &AppHandle) {
    match build_tray_menu(app) {
        Ok(menu) => {
            if let Some(tray) = app.tray_by_id("main") {
                if let Err(e) = tray.set_menu(Some(menu)) {
                    log::warn!("refresh_tray: 设置菜单失败: {}", e);
                }
            } else {
                log::warn!("refresh_tray: 未找到 id 为 main 的托盘实例");
            }
        }
        Err(e) => {
            log::error!("refresh_tray: 构建菜单失败: {}", e);
        }
    }
}
