mod config;
mod display_observer;
mod monitor;
mod tray;

use config::{AppConfig, ConfigManager, InputPreset};
use monitor::{get_monitors, switch_input, set_vcp, MonitorInfo, SwitchResult, VCP_BRIGHTNESS, VCP_CONTRAST, VCP_VOLUME, VCP_POWER_MODE};
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
fn cmd_set_brightness(monitor_index: usize, value: u16) -> Result<(), String> {
    set_vcp(monitor_index, VCP_BRIGHTNESS, value)
}

#[tauri::command]
fn cmd_set_contrast(monitor_index: usize, value: u16) -> Result<(), String> {
    set_vcp(monitor_index, VCP_CONTRAST, value)
}

#[tauri::command]
fn cmd_set_volume(monitor_index: usize, value: u16) -> Result<(), String> {
    set_vcp(monitor_index, VCP_VOLUME, value)
}

#[tauri::command]
fn cmd_set_power_mode(monitor_index: usize, mode: u8) -> Result<(), String> {
    set_vcp(monitor_index, VCP_POWER_MODE, mode as u16)
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

#[tauri::command]
fn cmd_save_preset(
    state: tauri::State<'_, Arc<ConfigManager>>,
    preset: InputPreset,
) -> Result<(), String> {
    let mut config = state.get();
    if let Some(existing) = config.presets.iter_mut().find(|p| p.name == preset.name) {
        *existing = preset;
    } else {
        config.presets.push(preset);
    }
    state.save(config)
}

#[tauri::command]
fn cmd_delete_preset(
    state: tauri::State<'_, Arc<ConfigManager>>,
    name: String,
) -> Result<(), String> {
    let mut config = state.get();
    config.presets.retain(|p| p.name != name);
    state.save(config)
}

#[tauri::command]
fn cmd_apply_preset(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<ConfigManager>>,
    name: String,
) -> Result<Vec<String>, String> {
    let config = state.get();
    let preset = config.presets.iter().find(|p| p.name == name)
        .ok_or_else(|| format!("预设 '{}' 不存在", name))?;

    let mut results = Vec::new();
    for (idx_str, &input_val) in &preset.inputs {
        let monitor_index: usize = idx_str.parse()
            .map_err(|_| format!("无效的显示器索引: {}", idx_str))?;
        match switch_input(monitor_index, input_val as u8) {
            Ok(r) => results.push(r.message),
            Err(e) => results.push(format!("显示器 #{} 切换失败: {}", monitor_index, e)),
        }
    }
    tray::refresh_tray(&app);
    Ok(results)
}

#[tauri::command]
fn cmd_save_monitor_order(
    state: tauri::State<'_, Arc<ConfigManager>>,
    order: Vec<String>,
) -> Result<(), String> {
    let mut config = state.get();
    config.monitor_order = order;
    state.save(config)
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
            cmd_set_brightness,
            cmd_set_contrast,
            cmd_set_volume,
            cmd_set_power_mode,
            cmd_get_config,
            cmd_save_config,
            cmd_save_preset,
            cmd_delete_preset,
            cmd_apply_preset,
            cmd_save_monitor_order,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("MonitorPilot 启动失败: {}", e);
            std::process::exit(1);
        });
}
