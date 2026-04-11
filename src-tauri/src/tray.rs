use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::monitor::{get_monitors, switch_input};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("MonitorPilot")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            handle_menu_event(app, id);
        })
        .build(app)?;

    Ok(())
}

fn build_tray_menu(
    app: &AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mut builder = MenuBuilder::new(app);

    match get_monitors() {
        Ok(monitors) if !monitors.is_empty() => {
            for monitor in &monitors {
                let submenu = build_monitor_submenu(app, monitor)?;
                builder = builder.item(&submenu);
            }
        }
        Ok(_) => {
            let item = MenuItemBuilder::with_id("no_monitors", "未检测到显示器")
                .enabled(false)
                .build(app)?;
            builder = builder.item(&item);
        }
        Err(e) => {
            let item = MenuItemBuilder::with_id("error", format!("错误: {}", e))
                .enabled(false)
                .build(app)?;
            builder = builder.item(&item);
        }
    }

    builder = builder.separator();

    let refresh = MenuItemBuilder::with_id("refresh", "刷新显示器列表").build(app)?;
    builder = builder.item(&refresh);

    let settings = MenuItemBuilder::with_id("settings", "设置...").build(app)?;
    builder = builder.item(&settings);

    builder = builder.separator();

    let quit = MenuItemBuilder::with_id("quit", "退出 MonitorPilot").build(app)?;
    builder = builder.item(&quit);

    Ok(builder.build()?)
}

fn build_monitor_submenu(
    app: &AppHandle,
    monitor: &crate::monitor::MonitorInfo,
) -> Result<tauri::menu::Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let mut submenu = SubmenuBuilder::new(app, &monitor.model);

    for input in &monitor.supported_inputs {
        let is_active = monitor.current_input == Some(input.value);
        let item_id = format!("switch_{}_{}", monitor.index, input.value);
        let label = if is_active {
            format!("✓ {}", input.name)
        } else {
            format!("   {}", input.name)
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
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "refresh" => {
            refresh_tray(app);
        }
        id if id.starts_with("switch_") => {
            let parts: Vec<&str> = id.split('_').collect();
            if parts.len() == 3 {
                if let (Ok(monitor_idx), Ok(input_val)) =
                    (parts[1].parse::<usize>(), parts[2].parse::<u8>())
                {
                    let _ = switch_input(monitor_idx, input_val);
                    refresh_tray(app);
                }
            }
        }
        _ => {}
    }
}

pub fn refresh_tray(app: &AppHandle) {
    if let Ok(menu) = build_tray_menu(app) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
