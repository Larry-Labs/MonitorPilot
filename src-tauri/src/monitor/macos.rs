use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::input_map::{input_name, is_known_input, supported_inputs_with_current};
use super::retry::{read_vcp_with_retry, write_vcp_with_retry, write_input_with_retry};
use super::types::{MonitorInfo, SwitchResult, VCP_BRIGHTNESS, VCP_CONTRAST, VCP_VOLUME, VCP_POWER_MODE};
use super::verify::{verify_switch, DdcOps};

const M1DDC_TIMEOUT: Duration = Duration::from_secs(5);
static M1DDC_PATH: OnceLock<String> = OnceLock::new();

struct MacOsDdc {
    display_num: u32,
}

impl DdcOps for MacOsDdc {
    fn read_input(&mut self) -> Option<u8> {
        get_input(self.display_num)
    }

    fn write_input(&mut self, value: u8) -> Result<(), String> {
        set_input(self.display_num, value)
    }

    fn read_vcp(&mut self, code: u8) -> Option<u16> {
        let attr = vcp_to_m1ddc_attr(code)?;
        let output = run_m1ddc(&["get", attr, "-d", &self.display_num.to_string()]).ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse::<u16>().ok()
    }

    fn write_vcp(&mut self, code: u8, value: u16) -> Result<(), String> {
        let attr = vcp_to_m1ddc_attr(code)
            .ok_or_else(|| format!("不支持的 VCP 代码: 0x{:02X}", code))?;
        let val_str = value.to_string();
        let disp_str = self.display_num.to_string();
        let output = run_m1ddc(&["set", attr, &val_str, "-d", &disp_str])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("DDC 写入失败: {}", stderr.trim()));
        }
        Ok(())
    }
}

fn vcp_to_m1ddc_attr(code: u8) -> Option<&'static str> {
    match code {
        0x10 => Some("luminance"),
        0x12 => Some("contrast"),
        0x62 => Some("volume"),
        0xD6 => Some("power"),
        _ => None,
    }
}

// --- m1ddc CLI helpers ---

fn find_m1ddc() -> &'static str {
    M1DDC_PATH.get_or_init(|| {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let sidecar = dir.join("m1ddc");
                if sidecar.exists() {
                    log::info!("使用内置 m1ddc: {}", sidecar.display());
                    return sidecar.to_string_lossy().to_string();
                }
            }
        }
        log::info!("使用系统 PATH 中的 m1ddc");
        "m1ddc".to_string()
    })
}

fn run_m1ddc(args: &[&str]) -> Result<std::process::Output, String> {
    let m1ddc = find_m1ddc();
    let mut child = Command::new(m1ddc)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法执行 m1ddc: {}。请确认已安装: brew install m1ddc", e))?;

    let deadline = Instant::now() + M1DDC_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_end(&mut stderr);
                }
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    log::error!(
                        "m1ddc 执行超时 ({}s)，已强制终止",
                        M1DDC_TIMEOUT.as_secs()
                    );
                    return Err("m1ddc 响应超时，请检查显示器连接".to_string());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("等待 m1ddc 退出失败: {}", e));
            }
        }
    }
}

// --- DDC read/write via m1ddc ---

fn get_input(display_num: u32) -> Option<u8> {
    let output = run_m1ddc(&["get", "input", "-d", &display_num.to_string()]).ok()?;

    if !output.status.success() {
        log::debug!(
            "读取显示器 #{} 输入失败: 退出码 {}",
            display_num,
            output.status
        );
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    let value = trimmed.parse::<u8>().ok().or_else(|| {
        trimmed.parse::<i32>().ok().and_then(|v| {
            if (0..=255).contains(&v) {
                Some(v as u8)
            } else {
                log::debug!("显示器 #{} 返回超范围值: {}，忽略", display_num, v);
                None
            }
        })
    });
    if value.is_none() {
        log::debug!("解析显示器 #{} 输入值失败: {:?}", display_num, trimmed);
    } else if let Some(v) = value {
        if !is_known_input(v) {
            log::debug!("显示器 #{} 返回非标准输入值: 0x{:02X}", display_num, v);
        }
    }
    value
}

fn set_input(display_num: u32, value: u8) -> Result<(), String> {
    let val_str = value.to_string();
    let disp_str = display_num.to_string();
    let output = run_m1ddc(&["set", "input", &val_str, "-d", &disp_str])?;

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
        return Err(msg);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.contains("Could not find")
        || trimmed.starts_with("Error")
        || trimmed.starts_with("error:")
    {
        return Err(trimmed.to_string());
    }

    Ok(())
}

// --- m1ddc output parsing ---

/// Parse a single m1ddc display list line.
/// Format: "[N] ModelName (UUID)" or "[N] (null) (UUID)"
/// Returns (display_num, model_name).
fn parse_m1ddc_line(line: &str) -> Option<(u32, String)> {
    let display_num: u32;
    let rest;

    if let Some(stripped) = line.strip_prefix('[') {
        if let Some(bracket_end) = stripped.find(']') {
            let num_str = stripped[..bracket_end].trim();
            display_num = match num_str.parse() {
                Ok(n) => n,
                Err(_) => {
                    log::warn!("m1ddc 显示器编号解析失败: {:?}，跳过此行", num_str);
                    return None;
                }
            };
            rest = stripped[bracket_end + 1..].trim();
        } else {
            log::warn!("m1ddc 输出格式异常（缺少 ']'），跳过: {:?}", line);
            return None;
        }
    } else {
        log::warn!("m1ddc 输出格式异常（缺少 '['），跳过: {:?}", line);
        return None;
    }

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

    Some((display_num, name))
}

// --- Public API ---

pub fn get_monitors() -> Result<Vec<MonitorInfo>, String> {
    let _guard = super::DDC_LOCK
        .lock()
        .map_err(|e| {
            log::error!("DDC_LOCK 获取失败 (poison={})", e);
            "DDC 内部错误，请重启应用".to_string()
        })?;

    let m1ddc = find_m1ddc();
    log::debug!("m1ddc 路径: {}", m1ddc);

    let output = run_m1ddc(&["display", "list"])?;

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

        let Some((display_num, model)) = parse_m1ddc_line(line) else {
            continue;
        };

        if model == "内置显示器" {
            log::debug!("跳过内置显示器: display_num={}", display_num);
            continue;
        }

        let current_input = get_input(display_num);
        log::info!(
            "检测到显示器: #{} {} | 当前输入: {}",
            display_num,
            model,
            current_input
                .map(input_name)
                .unwrap_or_else(|| "未知".to_string())
        );

        let mut ops = MacOsDdc { display_num };
        monitors.push(MonitorInfo {
            index: display_num as usize,
            model,
            current_input,
            current_input_name: current_input
                .map(input_name)
                .unwrap_or_else(|| "未知".to_string()),
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

    let display_num = monitor_index as u32;
    let mut ops = MacOsDdc { display_num };
    let previous_input = ops.read_input();

    log::info!(
        "切换请求: 显示器 #{} | {} → {} | 当前: {}",
        display_num,
        previous_input
            .map(input_name)
            .unwrap_or_else(|| "未知".to_string()),
        input_name(input_value),
        previous_input
            .map(|v| format!("0x{:02X}", v))
            .unwrap_or_else(|| "N/A".to_string())
    );

    write_input_with_retry(&mut ops, input_value).map_err(|e| {
        log::error!("切换失败: {}", e);
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

    let mut ops = MacOsDdc {
        display_num: monitor_index as u32,
    };
    write_vcp_with_retry(&mut ops, code, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_m1ddc_line_standard() {
        let (num, model) = parse_m1ddc_line("[1] LG ULTRAGEAR (ABC123)").unwrap();
        assert_eq!(num, 1);
        assert_eq!(model, "LG ULTRAGEAR");
    }

    #[test]
    fn parse_m1ddc_line_multi_digit_index() {
        let (num, model) = parse_m1ddc_line("[12] Dell U2723QE (UUID-123)").unwrap();
        assert_eq!(num, 12);
        assert_eq!(model, "Dell U2723QE");
    }

    #[test]
    fn parse_m1ddc_line_null_model_is_builtin() {
        let (num, model) = parse_m1ddc_line("[1] (null) (37D8832A-2D66)").unwrap();
        assert_eq!(num, 1);
        assert_eq!(model, "内置显示器");
    }

    #[test]
    fn parse_m1ddc_line_no_bracket_returns_none() {
        assert!(parse_m1ddc_line("Samsung LS27A (UUID)").is_none());
    }

    #[test]
    fn parse_m1ddc_line_empty_before_paren() {
        let (num, model) = parse_m1ddc_line("[2] (UUID-ONLY)").unwrap();
        assert_eq!(num, 2);
        assert_eq!(model, "内置显示器");
    }

    #[test]
    fn parse_m1ddc_line_no_uuid() {
        let (num, model) = parse_m1ddc_line("[3] Plain Model Name").unwrap();
        assert_eq!(num, 3);
        assert_eq!(model, "Plain Model Name");
    }
}
