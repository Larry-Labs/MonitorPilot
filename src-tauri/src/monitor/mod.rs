mod types;
mod input_map;
mod verify;
mod retry;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod desktop;

pub use types::{MonitorInfo, SwitchResult, VCP_BRIGHTNESS, VCP_CONTRAST, VCP_VOLUME, VCP_POWER_MODE};

#[cfg(target_os = "macos")]
pub use macos::{get_monitors, switch_input, set_vcp};
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use desktop::{get_monitors, switch_input, set_vcp};

use std::sync::Mutex;
pub(crate) static DDC_LOCK: Mutex<()> = Mutex::new(());
