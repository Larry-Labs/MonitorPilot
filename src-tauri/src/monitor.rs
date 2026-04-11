use serde::Serialize;
use std::process::Command;

#[cfg(not(target_os = "macos"))]
const VCP_INPUT_SOURCE: u8 = 0x60;

#[derive(Serialize, Clone, Debug)]
pub struct InputSource {
    pub value: u8,
    pub name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct MonitorInfo {
    pub index: usize,
    pub model: String,
    pub current_input: Option<u8>,
    pub current_input_name: String,
    pub supported_inputs: Vec<InputSource>,
}

fn input_name(value: u8) -> String {
    match value {
        0x01 => "VGA-1".to_string(),
        0x02 => "VGA-2".to_string(),
        0x03 => "DVI-1".to_string(),
        0x04 => "DVI-2".to_string(),
        0x0F => "DP-1".to_string(),
        0x10 => "DP-2".to_string(),
        0x11 => "HDMI-1".to_string(),
        0x12 => "HDMI-2".to_string(),
        0x13 => "HDMI-3".to_string(),
        0x14 => "HDMI-4".to_string(),
        v => format!("Input-0x{:02X}", v),
    }
}

fn default_supported_inputs() -> Vec<InputSource> {
    vec![0x0F, 0x10, 0x11, 0x12]
        .into_iter()
        .map(|v| InputSource {
            value: v,
            name: input_name(v),
        })
        .collect()
}

// --- macOS: use m1ddc CLI ---

#[cfg(target_os = "macos")]
pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let output = Command::new("m1ddc")
        .args(["display", "list"])
        .output()
        .map_err(|e| format!("无法执行 m1ddc: {}。请确认已安装: brew install m1ddc", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();

    for (i, line) in stdout.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let model = line.to_string();
        let display_num = (i + 1) as u32;
        let current_input = macos_get_input(display_num);

        monitors.push(MonitorInfo {
            index: display_num as usize,
            model,
            current_input,
            current_input_name: current_input
                .map(|v| input_name(v))
                .unwrap_or_else(|| "Unknown".to_string()),
            supported_inputs: default_supported_inputs(),
        });
    }

    Ok(monitors)
}

#[cfg(target_os = "macos")]
fn macos_get_input(display_num: u32) -> Option<u8> {
    let output = Command::new("m1ddc")
        .args(["get", "input", "-d", &display_num.to_string()])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    trimmed.parse::<u8>().ok()
}

#[cfg(target_os = "macos")]
pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<String, String> {
    let display_num = monitor_index as u32;

    let output = Command::new("m1ddc")
        .args([
            "set",
            "input",
            &input_value.to_string(),
            "-d",
            &display_num.to_string(),
        ])
        .output()
        .map_err(|e| format!("无法执行 m1ddc: {}", e))?;

    if output.status.success() {
        Ok(format!("已切换到 {}", input_name(input_value)))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("切换失败: {}", stderr))
    }
}

// --- Linux/Windows: use ddc-hi ---

#[cfg(not(target_os = "macos"))]
pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    use ddc_hi::{Ddc, Display};

    let displays = Display::enumerate();
    let mut monitors = Vec::new();

    for (i, mut display) in displays.into_iter().enumerate() {
        let model = display
            .info
            .model_name
            .clone()
            .unwrap_or_else(|| format!("Monitor {}", i + 1));

        let current_input = display
            .handle
            .get_vcp_feature(VCP_INPUT_SOURCE)
            .ok()
            .map(|v| v.value() as u8);

        let current_input_name = current_input
            .map(|v| input_name(v))
            .unwrap_or_else(|| "Unknown".to_string());

        monitors.push(MonitorInfo {
            index: i,
            model,
            current_input,
            current_input_name,
            supported_inputs: default_supported_inputs(),
        });
    }

    Ok(monitors)
}

#[cfg(not(target_os = "macos"))]
pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<String, String> {
    use ddc_hi::{Ddc, Display};

    let displays = Display::enumerate();
    let mut display = displays
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| format!("未找到 Monitor {}", monitor_index))?;

    display
        .handle
        .set_vcp_feature(VCP_INPUT_SOURCE, input_value as u16)
        .map_err(|e| format!("切换失败: {}", e))?;

    Ok(format!("已切换到 {}", input_name(input_value)))
}
