use ddc_hi::{Ddc, Display};
use serde::Serialize;

const VCP_INPUT_SOURCE: u8 = 0x60;

#[derive(Serialize, Clone)]
pub struct InputSource {
    pub value: u8,
    pub name: String,
}

#[derive(Serialize, Clone)]
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

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let displays = Display::enumerate();
    let mut monitors = Vec::new();

    for (i, mut display) in displays.into_iter().enumerate() {
        let model = display.info.model_name.clone().unwrap_or_else(|| format!("Monitor {}", i + 1));

        let current_input = display
            .handle
            .get_vcp_feature(VCP_INPUT_SOURCE)
            .ok()
            .map(|v| v.value() as u8);

        let current_input_name = current_input
            .map(|v| input_name(v))
            .unwrap_or_else(|| "Unknown".to_string());

        let supported_inputs = get_supported_inputs(&mut display);

        monitors.push(MonitorInfo {
            index: i,
            model,
            current_input,
            current_input_name,
            supported_inputs,
        });
    }

    Ok(monitors)
}

fn get_supported_inputs(_display: &mut Display) -> Vec<InputSource> {
    let common_inputs: Vec<u8> = vec![0x01, 0x03, 0x0F, 0x10, 0x11, 0x12];
    common_inputs
        .into_iter()
        .map(|v| InputSource {
            value: v,
            name: input_name(v),
        })
        .collect()
}

pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<String, String> {
    let displays = Display::enumerate();
    let mut display = displays
        .into_iter()
        .nth(monitor_index)
        .ok_or_else(|| format!("Monitor index {} not found", monitor_index))?;

    display
        .handle
        .set_vcp_feature(VCP_INPUT_SOURCE, input_value as u16)
        .map_err(|e| format!("Failed to switch input: {}", e))?;

    Ok(format!("Switched to {}", input_name(input_value)))
}
