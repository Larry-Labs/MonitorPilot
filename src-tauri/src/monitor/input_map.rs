use super::types::InputSource;

pub fn input_name(value: u8) -> String {
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
        0x1B => "USB-C".to_string(),
        v => format!("Input-0x{:02X}", v),
    }
}

pub fn is_known_input(value: u8) -> bool {
    matches!(
        value,
        0x01 | 0x02 | 0x03 | 0x04 | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x1B
    )
}

pub fn supported_inputs_with_current(current: Option<u8>) -> Vec<InputSource> {
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
            inputs.push(InputSource {
                value: cur,
                name: input_name(cur),
            });
        }
    }

    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_name_known_values() {
        assert_eq!(input_name(0x0F), "DP-1");
        assert_eq!(input_name(0x10), "DP-2");
        assert_eq!(input_name(0x11), "HDMI-1");
        assert_eq!(input_name(0x12), "HDMI-2");
        assert_eq!(input_name(0x01), "VGA-1");
        assert_eq!(input_name(0x03), "DVI-1");
        assert_eq!(input_name(0x13), "HDMI-3");
        assert_eq!(input_name(0x14), "HDMI-4");
    }

    #[test]
    fn input_name_unknown_value_formats_hex() {
        assert_eq!(input_name(0x6E), "Input-0x6E");
        assert_eq!(input_name(0xFF), "Input-0xFF");
        assert_eq!(input_name(0x00), "Input-0x00");
    }

    #[test]
    fn supported_inputs_default_list() {
        let inputs = supported_inputs_with_current(None);
        assert_eq!(inputs.len(), 4);
        assert_eq!(inputs[0].value, 0x0F);
        assert_eq!(inputs[0].name, "DP-1");
        assert_eq!(inputs[1].value, 0x10);
        assert_eq!(inputs[2].value, 0x11);
        assert_eq!(inputs[3].value, 0x12);
    }

    #[test]
    fn supported_inputs_with_known_current_no_duplicate() {
        let inputs = supported_inputs_with_current(Some(0x0F));
        assert_eq!(inputs.len(), 4);
        assert_eq!(inputs[0].value, 0x0F);
    }

    #[test]
    fn supported_inputs_with_unknown_current_appends() {
        let inputs = supported_inputs_with_current(Some(0x6E));
        assert_eq!(inputs.len(), 5);
        assert_eq!(inputs[0].value, 0x0F);
        assert_eq!(inputs[4].value, 0x6E);
        assert_eq!(inputs[4].name, "Input-0x6E");
    }

    #[test]
    fn is_known_input_validates_standard_values() {
        assert!(is_known_input(0x0F));
        assert!(is_known_input(0x10));
        assert!(is_known_input(0x11));
        assert!(is_known_input(0x12));
        assert!(is_known_input(0x1B));
        assert!(!is_known_input(0x63));
        assert!(!is_known_input(0x00));
        assert!(!is_known_input(0xFF));
    }
}
