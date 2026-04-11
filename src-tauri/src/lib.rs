mod monitor;
mod tray;

use monitor::{MonitorInfo, get_monitors, switch_input};
use serde::Serialize;

#[derive(Serialize)]
struct MonitorListResult {
    monitors: Vec<MonitorInfo>,
    error: Option<String>,
}

#[tauri::command]
fn cmd_get_monitors() -> MonitorListResult {
    match get_monitors() {
        Ok(monitors) => MonitorListResult { monitors, error: None },
        Err(e) => MonitorListResult { monitors: vec![], error: Some(e) },
    }
}

#[tauri::command]
fn cmd_switch_input(app: tauri::AppHandle, monitor_index: usize, input_value: u8) -> Result<String, String> {
    let result = switch_input(monitor_index, input_value);
    if result.is_ok() {
        tray::refresh_tray(&app);
    }
    result
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

            tray::setup_tray(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![cmd_get_monitors, cmd_switch_input])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
