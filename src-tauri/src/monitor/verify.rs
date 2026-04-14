use std::time::Duration;
use super::input_map::input_name;
use super::types::SwitchResult;

const VERIFY_DELAYS: &[u64] = &[600, 1400];
const POST_ROLLBACK_VERIFY_MS: u64 = 600;

pub(crate) trait DdcOps {
    fn read_input(&mut self) -> Option<u8>;
    fn write_input(&mut self, value: u8) -> Result<(), String>;
    fn read_vcp(&mut self, code: u8) -> Option<u16>;
    fn write_vcp(&mut self, code: u8, value: u16) -> Result<(), String>;
}

pub(crate) fn verify_switch(
    target_value: u8,
    previous_input: Option<u8>,
    ops: &mut dyn DdcOps,
) -> Result<SwitchResult, String> {
    log::debug!("切换命令已发送，开始多轮验证 ({}轮)", VERIFY_DELAYS.len());

    let mut confirmed = false;

    for (round, &delay) in VERIFY_DELAYS.iter().enumerate() {
        std::thread::sleep(Duration::from_millis(delay));

        match ops.read_input() {
            Some(actual) if actual == target_value => {
                log::debug!(
                    "验证第{}轮: 确认目标输入 {}",
                    round + 1,
                    input_name(target_value)
                );
                confirmed = true;
            }
            Some(actual) if confirmed => {
                log::debug!(
                    "验证第{}轮: DDC 抖动读到 {}，但已确认过目标 {}，忽略",
                    round + 1,
                    input_name(actual),
                    input_name(target_value)
                );
            }
            Some(actual) => {
                log::warn!(
                    "验证第{}轮: 期望 {} 实际 {} — 目标端口可能无信号",
                    round + 1,
                    input_name(target_value),
                    input_name(actual)
                );
                return Ok(SwitchResult {
                    status: "warning".to_string(),
                    message: format!(
                        "切换失败：{} 无信号，已自动恢复到 {}",
                        input_name(target_value),
                        input_name(actual)
                    ),
                    actual_input: Some(actual),
                });
            }
            None if round == 0 => {
                return attempt_rollback(target_value, previous_input, ops);
            }
            None => {
                log::debug!("验证第{}轮: DDC 暂时不可达，跳过", round + 1);
            }
        }
    }

    if confirmed {
        log::info!(
            "切换验证通过: → {} ({}轮)",
            input_name(target_value),
            VERIFY_DELAYS.len()
        );
        Ok(SwitchResult {
            status: "success".to_string(),
            message: format!("已切换到 {}", input_name(target_value)),
            actual_input: Some(target_value),
        })
    } else {
        log::warn!(
            "切换验证未能最终确认: 最后一轮 DDC 不可达",
        );
        Ok(SwitchResult {
            status: "warning".to_string(),
            message: format!(
                "已发送切换到 {} 的命令，但无法确认最终状态",
                input_name(target_value)
            ),
            actual_input: None,
        })
    }
}

fn attempt_rollback(
    target_value: u8,
    previous_input: Option<u8>,
    ops: &mut dyn DdcOps,
) -> Result<SwitchResult, String> {
    log::warn!(
        "切换后显示器不可达，尝试回滚到 {:?}",
        previous_input.map(input_name)
    );

    if let Some(prev) = previous_input {
        match ops.write_input(prev) {
            Ok(()) => {
                std::thread::sleep(Duration::from_millis(POST_ROLLBACK_VERIFY_MS));
                let current = ops.read_input();
                if current == Some(prev) {
                    log::info!("回滚成功: 已恢复到 {}", input_name(prev));
                    return Ok(SwitchResult {
                        status: "warning".to_string(),
                        message: format!(
                            "切换到 {} 后显示器失联，已自动恢复到 {}",
                            input_name(target_value),
                            input_name(prev)
                        ),
                        actual_input: Some(prev),
                    });
                }
                log::warn!(
                    "回滚命令已发送但结果不一致: 期望 {}, 实际 {:?}",
                    input_name(prev),
                    current.map(input_name)
                );
                if let Some(actual) = current {
                    return Ok(SwitchResult {
                        status: "warning".to_string(),
                        message: format!(
                            "切换到 {} 失败，显示器当前在 {}",
                            input_name(target_value),
                            input_name(actual)
                        ),
                        actual_input: Some(actual),
                    });
                }
                return Ok(SwitchResult {
                    status: "warning".to_string(),
                    message: format!(
                        "已尝试恢复到 {}，但无法确认最终状态",
                        input_name(prev)
                    ),
                    actual_input: None,
                });
            }
            Err(e) => log::error!("回滚命令失败: {}", e),
        }
    }

    Err(format!(
        "切换到 {} 后显示器失联（DDC/CI 通信中断），请检查线缆连接",
        input_name(target_value)
    ))
}
