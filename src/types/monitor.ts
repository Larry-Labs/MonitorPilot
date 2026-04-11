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
}

export interface MonitorListResult {
  monitors: MonitorInfo[];
  error: string | null;
}

export interface SwitchResult {
  status: "success" | "warning";
  message: string;
}

export interface AppConfig {
  input_names: Record<string, string>;
}
