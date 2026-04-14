use ddc_hi::{Ddc, Display};

use super::input_map::{input_name, supported_inputs_with_current};
use super::retry::{read_vcp_with_retry, write_vcp_with_retry, write_input_with_retry};
use super::types::{MonitorInfo, SwitchResult, VCP_BRIGHTNESS, VCP_CONTRAST, VCP_VOLUME, VCP_POWER_MODE};
use super::verify::{verify_switch, DdcOps};

const VCP_INPUT_SOURCE: u8 = 0x60;

struct DdcHiAdapter {
    display: Display,
}

impl DdcOps for DdcHiAdapter {
    fn read_input(&mut self) -> Option<u8> {
        self.display
            .handle
            .get_vcp_feature(VCP_INPUT_SOURCE)
            .ok()
            .map(|v| v.value() as u8)
    }

    fn write_input(&mut self, value: u8) -> Result<(), String> {
        self.display
            .handle
            .set_vcp_feature(VCP_INPUT_SOURCE, value as u16)
            .map_err(|e| format!("DDC 写入失败: {}", e))
    }

    fn read_vcp(&mut self, code: u8) -> Option<u16> {
        self.display
            .handle
            .get_vcp_feature(code)
            .ok()
            .map(|v| v.value())
    }

    fn write_vcp(&mut self, code: u8, value: u16) -> Result<(), String> {
        self.display
            .handle
            .set_vcp_feature(code, value)
            .map_err(|e| format!("DDC 写入 0x{:02X} 失败: {}", code, e))
    }
}

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let _guard = super::DDC_LOCK
        .lock()
        .map_err(|e| {
            log::error!("DDC_LOCK 获取失败 (poison={})", e);
            "DDC 内部错误，请重启应用".to_string()
        })?;

    log::debug!("正在枚举 DDC/CI 显示器 (ddc-hi)...");
    let displays = Display::enumerate();
    let mut monitors = Vec::new();

    for (i, display) in displays.into_iter().enumerate() {
        let model = display
            .info
            .model_name
            .clone()
            .unwrap_or_else(|| format!("Monitor {}", i + 1));

        let mut ops = DdcHiAdapter { display };
        let current_input = ops.read_input();

        let current_input_name = current_input
            .map(input_name)
            .unwrap_or_else(|| "未知".to_string());

        log::info!(
            "检测到显示器: #{} {} | 当前输入: {}",
            i,
            model,
            current_input_name
        );

        monitors.push(MonitorInfo {
            index: i,
            model,
            current_input,
            current_input_name,
            supported_inputs: supported_inputs_with_current(current_input),
            brightness: read_vcp_with_retry(&mut ops, VCP_BRIGHTNESS),
            contrast: read_vcp_with_retry(&mut ops, VCP_CONTRAST),
            volume: read_vcp_with_retry(&mut ops, VCP_VOLUME),
            power_mode: read_vcp_with_retry(&mut ops, VCP_POWER_MODE).map(|v| v as u8),
        });
    }

    log::info!("共检测到 {} 台外接显示器", monitors.len());
    Ok(monitors)
}

pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<SwitchResult, String> {
    let _guard = super::DDC_LOCK
        .lock()
        .map_err(|e| {
            log::error!("DDC_LOCK 获取失败 (poison={})", e);
            "DDC 内部错误，请重启应用".to_string()
        })?;

    let displays = Display::enumerate();
    let display = displays.into_iter().nth(monitor_index).ok_or_else(|| {
        log::error!("未找到显示器 #{}", monitor_index);
        format!("切换失败: 未找到显示器 #{}", monitor_index)
    })?;

    let mut ops = DdcHiAdapter { display };
    let previous_input = ops.read_input();

    log::info!(
        "切换请求: 显示器 #{} | {} → {} | 当前: {}",
        monitor_index,
        previous_input
            .map(input_name)
            .unwrap_or_else(|| "未知".to_string()),
        input_name(input_value),
        previous_input
            .map(|v| format!("0x{:02X}", v))
            .unwrap_or_else(|| "N/A".to_string())
    );

    write_input_with_retry(&mut ops, input_value).map_err(|e| {
        log::error!(
            "切换失败: 显示器 #{} → {} | {}",
            monitor_index,
            input_name(input_value),
            e
        );
        format!("切换失败: {}", e)
    })?;

    verify_switch(input_value, previous_input, &mut ops)
}

pub fn set_vcp(monitor_index: usize, code: u8, value: u16) -> Result<(), String> {
    let _guard = super::DDC_LOCK
        .lock()
        .map_err(|e| {
            log::error!("DDC_LOCK 获取失败 (poison={})", e);
            "DDC 内部错误，请重启应用".to_string()
        })?;

    let displays = Display::enumerate();
    let display = displays.into_iter().nth(monitor_index).ok_or_else(|| {
        format!("未找到显示器 #{}", monitor_index)
    })?;

    let mut ops = DdcHiAdapter { display };
    write_vcp_with_retry(&mut ops, code, value)
}
