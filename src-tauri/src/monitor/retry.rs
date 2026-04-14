use std::time::Duration;
use super::verify::DdcOps;

const WRITE_MAX_RETRIES: u32 = 3;
const READ_MAX_RETRIES: u32 = 4;
const WRITE_BASE_DELAY_MS: u64 = 50;
const READ_BASE_DELAY_MS: u64 = 40;

fn retry_write<F>(mut action: F, max_retries: u32, label: &str) -> Result<(), String>
where
    F: FnMut() -> Result<(), String>,
{
    let mut last_err = String::new();
    for attempt in 0..=max_retries {
        match action() {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                if attempt < max_retries {
                    let delay = WRITE_BASE_DELAY_MS * (attempt as u64 + 1);
                    log::debug!("{}（第{}次），{}ms 后重试", label, attempt + 1, delay);
                    std::thread::sleep(Duration::from_millis(delay));
                }
            }
        }
    }
    Err(format!("{}（共尝试 {} 次均失败）: {}", label, max_retries + 1, last_err))
}

pub(crate) fn write_vcp_with_retry(
    ops: &mut dyn DdcOps,
    code: u8,
    value: u16,
) -> Result<(), String> {
    let label = format!("VCP 0x{:02X} 写入失败", code);
    retry_write(|| ops.write_vcp(code, value), WRITE_MAX_RETRIES, &label)
}

pub(crate) fn read_vcp_with_retry(ops: &mut dyn DdcOps, code: u8) -> Option<u16> {
    for attempt in 0..=READ_MAX_RETRIES {
        if let Some(val) = ops.read_vcp(code) {
            return Some(val);
        }
        if attempt < READ_MAX_RETRIES {
            let delay = READ_BASE_DELAY_MS * (attempt as u64 + 1);
            log::debug!(
                "VCP 0x{:02X} 读取失败（第{}次），{}ms 后重试",
                code,
                attempt + 1,
                delay
            );
            std::thread::sleep(Duration::from_millis(delay));
        }
    }
    log::warn!(
        "VCP 0x{:02X} 读取失败（共尝试 {} 次均失败）",
        code,
        READ_MAX_RETRIES + 1
    );
    None
}

pub(crate) fn write_input_with_retry(
    ops: &mut dyn DdcOps,
    value: u8,
) -> Result<(), String> {
    retry_write(|| ops.write_input(value), WRITE_MAX_RETRIES, "输入源切换失败")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDdc {
        write_fail_count: u32,
        read_fail_count: u32,
        write_calls: u32,
        read_calls: u32,
    }

    impl DdcOps for MockDdc {
        fn read_input(&mut self) -> Option<u8> {
            None
        }
        fn write_input(&mut self, _value: u8) -> Result<(), String> {
            self.write_calls += 1;
            if self.write_calls <= self.write_fail_count {
                Err("mock write error".to_string())
            } else {
                Ok(())
            }
        }
        fn read_vcp(&mut self, _code: u8) -> Option<u16> {
            self.read_calls += 1;
            if self.read_calls <= self.read_fail_count {
                None
            } else {
                Some(50)
            }
        }
        fn write_vcp(&mut self, _code: u8, _value: u16) -> Result<(), String> {
            self.write_calls += 1;
            if self.write_calls <= self.write_fail_count {
                Err("mock write error".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn write_vcp_succeeds_first_try() {
        let mut mock = MockDdc {
            write_fail_count: 0,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        assert!(write_vcp_with_retry(&mut mock, 0x10, 50).is_ok());
        assert_eq!(mock.write_calls, 1);
    }

    #[test]
    fn write_vcp_succeeds_after_retries() {
        let mut mock = MockDdc {
            write_fail_count: 2,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        assert!(write_vcp_with_retry(&mut mock, 0x10, 50).is_ok());
        assert_eq!(mock.write_calls, 3);
    }

    #[test]
    fn write_vcp_fails_after_max_retries() {
        let mut mock = MockDdc {
            write_fail_count: 10,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        let result = write_vcp_with_retry(&mut mock, 0x10, 50);
        assert!(result.is_err());
        assert_eq!(mock.write_calls, WRITE_MAX_RETRIES + 1);
    }

    #[test]
    fn read_vcp_succeeds_first_try() {
        let mut mock = MockDdc {
            write_fail_count: 0,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        assert_eq!(read_vcp_with_retry(&mut mock, 0x10), Some(50));
        assert_eq!(mock.read_calls, 1);
    }

    #[test]
    fn read_vcp_succeeds_after_retries() {
        let mut mock = MockDdc {
            write_fail_count: 0,
            read_fail_count: 3,
            write_calls: 0,
            read_calls: 0,
        };
        assert_eq!(read_vcp_with_retry(&mut mock, 0x10), Some(50));
        assert_eq!(mock.read_calls, 4);
    }

    #[test]
    fn read_vcp_returns_none_after_max_retries() {
        let mut mock = MockDdc {
            write_fail_count: 0,
            read_fail_count: 10,
            write_calls: 0,
            read_calls: 0,
        };
        assert_eq!(read_vcp_with_retry(&mut mock, 0x10), None);
        assert_eq!(mock.read_calls, READ_MAX_RETRIES + 1);
    }

    #[test]
    fn write_input_succeeds_after_retries() {
        let mut mock = MockDdc {
            write_fail_count: 1,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        assert!(write_input_with_retry(&mut mock, 0x0f).is_ok());
        assert_eq!(mock.write_calls, 2);
    }

    #[test]
    fn write_input_fails_after_max_retries() {
        let mut mock = MockDdc {
            write_fail_count: 10,
            read_fail_count: 0,
            write_calls: 0,
            read_calls: 0,
        };
        let result = write_input_with_retry(&mut mock, 0x0f);
        assert!(result.is_err());
        assert_eq!(mock.write_calls, WRITE_MAX_RETRIES + 1);
    }
}
