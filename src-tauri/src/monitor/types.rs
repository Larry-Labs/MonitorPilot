use serde::Serialize;

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
    pub brightness: Option<u16>,
    pub contrast: Option<u16>,
    pub volume: Option<u16>,
    pub power_mode: Option<u8>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SwitchResult {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_input: Option<u8>,
}

pub const VCP_BRIGHTNESS: u8 = 0x10;
pub const VCP_CONTRAST: u8 = 0x12;
pub const VCP_VOLUME: u8 = 0x62;
pub const VCP_POWER_MODE: u8 = 0xD6;
