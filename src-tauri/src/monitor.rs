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

fn supported_inputs_with_current(current: Option<u8>) -> Vec<InputSource> {
    let defaults: Vec<u8> = vec![0x0F, 0x10, 0x11, 0x12];
    let mut inputs: Vec<InputSource> = defaults
        .iter()
        .map(|&v| InputSource {
            value: v,
            name: input_name(v),
        })
        .collect();

    if let Some(cur) = current {
        if !defaults.contains(&cur) {
            inputs.insert(
                0,
                InputSource {
                    value: cur,
                    name: input_name(cur),
                },
            );
        }
    }

    inputs
}

// --- macOS: use m1ddc CLI ---

#[cfg(target_os = "macos")]
fn find_m1ddc() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sidecar = dir.join("m1ddc");
            if sidecar.exists() {
                return sidecar.to_string_lossy().to_string();
            }
        }
    }
    "m1ddc".to_string()
}

#[cfg(target_os = "macos")]
pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let m1ddc = find_m1ddc();
    log::debug!("m1ddc 路径: {}", m1ddc);

    let output = Command::new(&m1ddc)
        .args(["display", "list"])
        .output()
        .map_err(|e| format!("无法执行 m1ddc: {}。请确认已安装: brew install m1ddc", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("m1ddc display list 失败: {}", stderr);
        return Err(format!("m1ddc display list 失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    log::debug!("m1ddc display list 输出: {}", stdout.trim());
    let mut monitors = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (display_num, model) = parse_m1ddc_line(line);

        if model == "内置显示器" {
            log::debug!("跳过内置显示器: display_num={}", display_num);
            continue;
        }

        let current_input = macos_get_input(display_num);
        log::info!(
            "检测到显示器: #{} {} | 当前输入: {}",
            display_num,
            model,
            current_input
                .map(|v| input_name(v))
                .unwrap_or_else(|| "未知".to_string())
        );

        monitors.push(MonitorInfo {
            index: display_num as usize,
            model,
            current_input,
            current_input_name: current_input
                .map(|v| input_name(v))
                .unwrap_or_else(|| "未知".to_string()),
            supported_inputs: supported_inputs_with_current(current_input),
        });
    }

    log::info!("共检测到 {} 台外接显示器", monitors.len());
    Ok(monitors)
}

/// Parse a single m1ddc display list line.
/// Format: "[N] ModelName (UUID)" or "[N] (null) (UUID)"
/// Returns (display_num, model_name).
#[cfg(target_os = "macos")]
fn parse_m1ddc_line(line: &str) -> (u32, String) {
    let mut display_num: u32 = 1;
    let rest;

    if let Some(stripped) = line.strip_prefix('[') {
        if let Some(bracket_end) = stripped.find(']') {
            display_num = stripped[..bracket_end].trim().parse().unwrap_or(1);
            rest = stripped[bracket_end + 1..].trim();
        } else {
            rest = line;
        }
    } else {
        rest = line;
    }

    // Remove trailing UUID in parentheses: "ModelName (UUID)" → "ModelName"
    let name = if let Some(idx) = rest.rfind('(') {
        let before = rest[..idx].trim();
        if before.is_empty() || before == "(null)" {
            "内置显示器".to_string()
        } else {
            before.to_string()
        }
    } else if rest.is_empty() || rest.contains("(null)") {
        "内置显示器".to_string()
    } else {
        rest.to_string()
    };

    (display_num, name)
}

#[cfg(target_os = "macos")]
fn macos_get_input(display_num: u32) -> Option<u8> {
    let m1ddc = find_m1ddc();
    let output = Command::new(&m1ddc)
        .args(["get", "input", "-d", &display_num.to_string()])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<u8>().ok()
}

#[cfg(target_os = "macos")]
pub fn switch_input(monitor_index: usize, input_value: u8) -> Result<String, String> {
    let m1ddc = find_m1ddc();
    let display_num = monitor_index as u32;

    let previous_input = macos_get_input(display_num);
    log::info!(
        "切换请求: 显示器 #{} | {} → {} | 当前: {}",
        display_num,
        previous_input
            .map(|v| input_name(v))
            .unwrap_or_else(|| "未知".to_string()),
        input_name(input_value),
        previous_input
            .map(|v| format!("0x{:02X}", v))
            .unwrap_or_else(|| "N/A".to_string())
    );

    let output = Command::new(&m1ddc)
        .args([
            "set",
            "input",
            &input_value.to_string(),
            "-d",
            &display_num.to_string(),
        ])
        .output()
        .map_err(|e| {
            log::error!("m1ddc 执行失败: {}", e);
            format!("无法执行 m1ddc: {}", e)
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if !stdout.trim().is_empty() {
            stdout.trim().to_string()
        } else if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            format!("m1ddc 退出码: {}", output.status)
        };
        log::error!("切换命令失败: {}", msg);
        return Err(format!("切换失败: {}", msg));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.contains("Could not find") || trimmed.contains("error") {
        log::error!("m1ddc 返回错误: {}", trimmed);
        return Err(format!("切换失败: {}", trimmed));
    }

    log::debug!("切换命令已发送，等待 500ms 验证...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    match macos_get_input(display_num) {
        Some(actual) if actual == input_value => {
            log::info!("切换成功: 显示器 #{} → {}", display_num, input_name(input_value));
            Ok(format!("已切换到 {}", input_name(input_value)))
        }
        Some(actual) => {
            log::warn!(
                "切换未生效: 目标 {} 实际 {} — 目标端口可能无信号",
                input_name(input_value),
                input_name(actual)
            );
            Ok(format!(
                "已发送切换指令到 {}，但显示器当前仍为 {}（目标端口可能无信号）",
                input_name(input_value),
                input_name(actual)
            ))
        }
        None => {
            log::warn!(
                "切换后显示器不可达，尝试回滚到 {:?}",
                previous_input.map(|v| input_name(v))
            );
            if let Some(prev) = previous_input {
                let rollback = Command::new(&m1ddc)
                    .args([
                        "set",
                        "input",
                        &prev.to_string(),
                        "-d",
                        &display_num.to_string(),
                    ])
                    .output();

                match rollback {
                    Ok(r) if r.status.success() => {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let recovered = macos_get_input(display_num);
                        if recovered == Some(prev) {
                            log::info!("回滚成功: 已恢复到 {}", input_name(prev));
                            return Err(format!(
                                "切换到 {} 后显示器失联，已自动恢复到 {}",
                                input_name(input_value),
                                input_name(prev)
                            ));
                        }
                        log::warn!("回滚命令已发送但无法确认结果");
                    }
                    Ok(_) => log::error!("回滚命令执行失败"),
                    Err(e) => log::error!("回滚命令发送失败: {}", e),
                }
            }
            Err(format!(
                "切换到 {} 后显示器失联（DDC/CI 通信中断），请检查线缆连接",
                input_name(input_value)
            ))
        }
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
            .unwrap_or_else(|| "未知".to_string());

        monitors.push(MonitorInfo {
            index: i,
            model,
            current_input,
            current_input_name,
            supported_inputs: supported_inputs_with_current(current_input),
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
