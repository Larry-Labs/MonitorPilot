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
            Some(actual) if super::input_map::canonical_input(actual) == super::input_map::canonical_input(target_value) => {
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
            Some(actual) if !super::input_map::is_known_input(actual) => {
                log::debug!(
                    "验证第{}轮: 读到无效值 0x{:02X}，视为过渡态，跳过",
                    round + 1,
                    actual
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
        // All rounds returned invalid/unknown values — actual state unknown.
        // Attempt rollback instead of falsely claiming "已恢复".
        log::warn!(
            "切换验证未能最终确认: 所有轮次均返回无效值，尝试回滚",
        );
        attempt_rollback(target_value, previous_input, ops)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockDdc {
        reads: RefCell<Vec<Option<u8>>>,
        writes: RefCell<Vec<u8>>,
        write_result: Result<(), String>,
    }

    impl MockDdc {
        fn new(reads: Vec<Option<u8>>) -> Self {
            Self {
                reads: RefCell::new(reads),
                writes: RefCell::new(Vec::new()),
                write_result: Ok(()),
            }
        }

        fn with_write_error(reads: Vec<Option<u8>>, err: &str) -> Self {
            Self {
                reads: RefCell::new(reads),
                writes: RefCell::new(Vec::new()),
                write_result: Err(err.to_string()),
            }
        }
    }

    impl DdcOps for MockDdc {
        fn read_input(&mut self) -> Option<u8> {
            let mut reads = self.reads.borrow_mut();
            if reads.is_empty() {
                None
            } else {
                reads.remove(0)
            }
        }

        fn write_input(&mut self, value: u8) -> Result<(), String> {
            self.writes.borrow_mut().push(value);
            self.write_result.clone()
        }

        fn read_vcp(&mut self, _code: u8) -> Option<u16> {
            None
        }

        fn write_vcp(&mut self, _code: u8, _value: u16) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn verify_success_both_rounds_confirm() {
        let mut ops = MockDdc::new(vec![Some(0x11), Some(0x11)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(result.actual_input, Some(0x11));
    }

    #[test]
    fn verify_success_first_round_confirms_second_jitters() {
        // Round 0: target confirmed, Round 1: DDC jitter reads different value
        let mut ops = MockDdc::new(vec![Some(0x11), Some(0x0F)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "success");
    }

    #[test]
    fn verify_success_vendor_code_equivalence() {
        // Monitor reports 0x6E (vendor DP) when target is 0x0F (standard DP)
        let mut ops = MockDdc::new(vec![Some(0x6E), Some(0x6E)]);
        let result = verify_switch(0x0F, Some(0x11), &mut ops).unwrap();
        assert_eq!(result.status, "success");
        assert!(result.message.contains("DP-1"));
    }

    #[test]
    fn verify_warning_different_known_input() {
        // Target is HDMI-1, but monitor reads back DP-1 (no signal on HDMI)
        let mut ops = MockDdc::new(vec![Some(0x0F)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("HDMI-1"));
        assert!(result.message.contains("DP-1"));
        assert_eq!(result.actual_input, Some(0x0F));
    }

    #[test]
    fn verify_invalid_value_treated_as_transient() {
        // Both rounds return 0x00 (invalid) → not confirmed → triggers rollback
        // Rollback write 0x0F → read confirms 0x0F
        let mut ops = MockDdc::new(vec![Some(0x00), Some(0x00), Some(0x0F)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("恢复到"));
        assert_eq!(result.actual_input, Some(0x0F));
        // Rollback actually wrote the previous input
        assert_eq!(*ops.writes.borrow(), vec![0x0F]);
    }

    #[test]
    fn verify_invalid_then_target_confirms() {
        // Round 0: invalid 0x00 (skip), Round 1: target confirmed
        let mut ops = MockDdc::new(vec![Some(0x00), Some(0x11)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(result.actual_input, Some(0x11));
    }

    #[test]
    fn verify_none_round0_triggers_rollback() {
        // Round 0: None → attempt rollback → write previous → read confirms
        let mut ops = MockDdc::new(vec![None, Some(0x0F)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("失联"));
        assert!(result.message.contains("恢复到"));
        assert_eq!(result.actual_input, Some(0x0F));
        assert_eq!(*ops.writes.borrow(), vec![0x0F]);
    }

    #[test]
    fn verify_none_round0_no_previous_errors() {
        let mut ops = MockDdc::new(vec![None]);
        let result = verify_switch(0x11, None, &mut ops);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("失联"));
    }

    #[test]
    fn verify_none_later_rounds_not_confirmed() {
        // Round 0: target confirmed, Round 1: None (DDC unreachable)
        let mut ops = MockDdc::new(vec![Some(0x11), None]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        // Still success because round 0 confirmed
        assert_eq!(result.status, "success");
    }

    #[test]
    fn verify_all_none_after_round0_not_confirmed() {
        // Round 0: invalid, Round 1: None → not confirmed → triggers rollback
        // Rollback write 0x0F → read returns None (can't confirm)
        let mut ops = MockDdc::new(vec![Some(0x00), None]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("无法确认"));
        assert_eq!(result.actual_input, None);
        assert_eq!(*ops.writes.borrow(), vec![0x0F]);
    }

    // --- Rollback scenarios ---

    #[test]
    fn rollback_write_fails_returns_error() {
        // Round 0: None → rollback → write fails → error
        let mut ops = MockDdc::with_write_error(vec![None], "DDC write timeout");
        let result = verify_switch(0x11, Some(0x0F), &mut ops);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("失联"));
    }

    #[test]
    fn rollback_write_ok_but_readback_different() {
        // Round 0: None → rollback → write OK → read back 0x12 (unexpected)
        let mut ops = MockDdc::new(vec![None, Some(0x12)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("HDMI-2"));
        assert_eq!(result.actual_input, Some(0x12));
    }

    #[test]
    fn rollback_write_ok_but_readback_none() {
        // Round 0: None → rollback → write OK → read None
        let mut ops = MockDdc::new(vec![None, None]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("无法确认"));
        assert_eq!(result.actual_input, None);
    }

    // --- Edge cases ---

    #[test]
    fn verify_multiple_invalid_values_all_skipped() {
        // Both rounds return different invalid values → triggers rollback
        // Rollback write 0x0F → read confirms 0x0F
        let mut ops = MockDdc::new(vec![Some(0x63), Some(0xFE), Some(0x0F)]);
        let result = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        assert_eq!(result.status, "warning");
        assert!(result.message.contains("恢复到"));
        assert_eq!(*ops.writes.borrow(), vec![0x0F]);
    }

    #[test]
    fn verify_target_is_vendor_code_matches_standard() {
        // Target is 0x6E (vendor DP), monitor reads back 0x0F (standard DP)
        let mut ops = MockDdc::new(vec![Some(0x0F), Some(0x0F)]);
        let result = verify_switch(0x6E, Some(0x11), &mut ops).unwrap();
        assert_eq!(result.status, "success");
    }

    #[test]
    fn verify_no_writes_on_successful_switch() {
        let mut ops = MockDdc::new(vec![Some(0x11), Some(0x11)]);
        let _ = verify_switch(0x11, Some(0x0F), &mut ops).unwrap();
        // No write_input calls during verification (only reads)
        assert!(ops.writes.borrow().is_empty());
    }
}
