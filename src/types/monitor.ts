export interface InputSource {
  value: number;
  name: string;
}

export interface MonitorInfo {
  index: number;
  model: string;
  current_input: number | null;
  current_input_name: string;
  supported_inputs: InputSource[];
  brightness: number | null;
  contrast: number | null;
  volume: number | null;
  power_mode: number | null;
}

export interface MonitorListResult {
  monitors: MonitorInfo[];
  error: string | null;
}

export interface SwitchResult {
  status: "success" | "warning";
  message: string;
  actual_input?: number;
}

export interface InputPreset {
  name: string;
  inputs: Record<string, number>;
}

export interface AppConfig {
  input_names: Record<string, string>;
  monitor_order: string[];
  presets: InputPreset[];
}
