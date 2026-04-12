mod types;
mod input_map;
mod verify;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod desktop;

pub use types::{MonitorInfo, SwitchResult};

#[cfg(target_os = "macos")]
pub use macos::{get_monitors, switch_input};
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use desktop::{get_monitors, switch_input};

use std::sync::Mutex;
pub(crate) static DDC_LOCK: Mutex<()> = Mutex::new(());
