mod config;
mod display_observer;
mod monitor;
mod tray;

use config::{AppConfig, ConfigManager};
use monitor::{get_monitors, switch_input, MonitorInfo, SwitchResult};
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
) -> Result<SwitchResult, String> {
    let result = switch_input(monitor_index, input_value);
    tray::refresh_tray(&app);
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
    state.save(config).map_err(|e| {
        log::error!("配置保存失败: {}", e);
        e
    }).map(|_| {
        log::debug!("配置已保存");
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    log::warn!("单实例回调: 显示窗口失败: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    log::warn!("单实例回调: 聚焦窗口失败: {}", e);
                }
            }
        }))
        .setup(|app| {
            let log_level = if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            };
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log_level)
                    .build(),
            )?;

            log::info!(
                "MonitorPilot v{} 启动 | {} | {}",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH
            );

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
            log::debug!("配置目录: {}", app_data_dir.display());
            let config_manager = Arc::new(ConfigManager::new(app_data_dir));
            app.manage(config_manager);

            tray::setup_tray(app.handle())?;
            display_observer::start(app.handle());

            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    log::warn!("启动时显示窗口失败: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    log::warn!("启动时聚焦窗口失败: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_get_monitors,
            cmd_switch_input,
            cmd_get_config,
            cmd_save_config,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("MonitorPilot 启动失败: {}", e);
            std::process::exit(1);
        });
}
