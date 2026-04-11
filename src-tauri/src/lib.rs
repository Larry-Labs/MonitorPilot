mod config;
mod monitor;
mod tray;

use config::{AppConfig, ConfigManager, HotkeyBinding};
use monitor::{get_monitors, switch_input, MonitorInfo};
use serde::Serialize;
use tauri::Manager;
use std::sync::Arc;

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
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<ConfigManager>>,
    config: AppConfig,
) -> Result<(), String> {
    state.save(config.clone())?;
    register_hotkeys_from_config(&app, &config.hotkeys);
    Ok(())
}

#[tauri::command]
fn cmd_save_hotkeys(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<ConfigManager>>,
    hotkeys: Vec<HotkeyBinding>,
) -> Result<(), String> {
    state.set_hotkeys(hotkeys.clone())?;
    register_hotkeys_from_config(&app, &hotkeys);
    Ok(())
}

fn register_hotkeys_from_config(app: &tauri::AppHandle, hotkeys: &[HotkeyBinding]) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let _ = app.global_shortcut().unregister_all();

    for binding in hotkeys {
        let monitor_idx = binding.monitor_index;
        let input_val = binding.input_value;
        let app_handle = app.clone();

        if let Ok(shortcut) = binding.shortcut.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                let _ = switch_input(monitor_idx, input_val);
                tray::refresh_tray(&app_handle);
            });
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let config_manager = Arc::new(ConfigManager::new(app_data_dir));

            let initial_hotkeys = config_manager.get().hotkeys.clone();
            app.manage(config_manager);

            tray::setup_tray(app.handle())?;

            if !initial_hotkeys.is_empty() {
                register_hotkeys_from_config(app.handle(), &initial_hotkeys);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_get_monitors,
            cmd_switch_input,
            cmd_get_config,
            cmd_save_config,
            cmd_save_hotkeys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
