#[cfg(target_os = "macos")]
mod macos {
    use std::sync::OnceLock;
    use tauri::{AppHandle, Emitter};

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

    const KCG_DISPLAY_ADD_FLAG: u32 = 1 << 4;
    const KCG_DISPLAY_REMOVE_FLAG: u32 = 1 << 5;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGDisplayRegisterReconfigurationCallback(
            callback: extern "C" fn(u32, u32, *mut std::ffi::c_void),
            user_data: *mut std::ffi::c_void,
        ) -> i32;
    }

    extern "C" fn on_display_change(
        _display: u32,
        flags: u32,
        _user_info: *mut std::ffi::c_void,
    ) {
        if flags & (KCG_DISPLAY_ADD_FLAG | KCG_DISPLAY_REMOVE_FLAG) != 0 {
            log::info!(
                "显示器 {} (flags: 0x{:X})",
                if flags & KCG_DISPLAY_ADD_FLAG != 0 { "已连接" } else { "已断开" },
                flags
            );
            if let Some(app) = APP_HANDLE.get() {
                if let Err(e) = app.emit("display-changed", ()) {
                    log::warn!("发送显示器变化事件失败: {}", e);
                }
            }
        }
    }

    pub fn start(app: &AppHandle) {
        if APP_HANDLE.set(app.clone()).is_err() {
            log::debug!("显示器监听已注册，跳过重复注册");
            return;
        }
        unsafe {
            let result = CGDisplayRegisterReconfigurationCallback(
                on_display_change,
                std::ptr::null_mut(),
            );
            if result == 0 {
                log::info!("macOS 显示器变化原生监听已注册");
            } else {
                log::error!("注册 CGDisplayReconfigurationCallback 失败: {}", result);
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod fallback {
    use tauri::AppHandle;

    pub fn start(_app: &AppHandle) {
        log::info!("当前平台未启用原生显示器变化监听，依赖前端轮询检测");
    }
}

#[cfg(target_os = "macos")]
pub use macos::start;

#[cfg(not(target_os = "macos"))]
pub use fallback::start;
