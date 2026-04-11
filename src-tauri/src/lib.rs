mod config;
mod monitor;
mod tray;

use config::{AppConfig, ConfigManager};
use monitor::{get_monitors, switch_input, MonitorInfo};
use serde::Serialize;
use std::sync::Arc;
use tauri::Manager;

#[derive(Serialize)]
struct MonitorListResult {
    monitors: Vec<MonitorInfo>,
    error: Option<String>,
}

#[tauri::command]
fn cmd_get_monitors() -> MonitorListResult {
    match get_monitors() {
        Ok(monitors) => MonitorListResult {
            monitors,
            error: None,
        },
        Err(e) => MonitorListResult {
            monitors: vec![],
            error: Some(e),
        },
    }
}

#[tauri::command]
fn cmd_switch_input(
    app: tauri::AppHandle,
    monitor_index: usize,
    input_value: u8,
) -> Result<String, String> {
    let result = switch_input(monitor_index, input_value);
    if result.is_ok() {
        tray::refresh_tray(&app);
    }
    result
}

#[tauri::command]
fn cmd_get_config(state: tauri::State<'_, Arc<ConfigManager>>) -> AppConfig {
    state.get()
}

#[tauri::command]
fn cmd_save_config(
    state: tauri::State<'_, Arc<ConfigManager>>,
    config: AppConfig,
) -> Result<(), String> {
    state.save(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
            let config_manager = Arc::new(ConfigManager::new(app_data_dir));
            app.manage(config_manager);

            tray::setup_tray(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_get_monitors,
            cmd_switch_input,
            cmd_get_config,
            cmd_save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
