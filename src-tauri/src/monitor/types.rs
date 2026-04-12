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
}

#[derive(Serialize, Clone, Debug)]
pub struct SwitchResult {
    pub status: String,
    pub message: String,
}
