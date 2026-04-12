use ddc_hi::{Ddc, Display};

use super::input_map::{input_name, supported_inputs_with_current};
use super::types::{MonitorInfo, SwitchResult};
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
}

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    log::debug!("正在枚举 DDC/CI 显示器 (ddc-hi)...");
    let displays = Display::enumerate();
    let mut monitors = Vec::new();

    for (i, mut display) in displays.into_iter().enumerate() {
        let model = display
            .info
            .model_name
            .clone()
            .unwrap_or_else(|| format!("Monitor {}", i + 1));

        let current_input = match display.handle.get_vcp_feature(VCP_INPUT_SOURCE) {
            Ok(v) => {
                let val = v.value() as u8;
                log::debug!("显示器 #{} 当前输入: 0x{:02X}", i, val);
                Some(val)
            }
            Err(e) => {
                log::debug!("读取显示器 #{} 输入失败: {}", i, e);
                None
            }
        };

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
        });
    }

    log::info!("共检测到 {} 台外接显示器", monitors.len());
    Ok(monitors)
}

pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<SwitchResult, String> {
    let _guard = super::DDC_LOCK
        .lock()
        .map_err(|_| "DDC 操作正忙，请稍后重试".to_string())?;

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

    ops.write_input(input_value).map_err(|e| {
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
