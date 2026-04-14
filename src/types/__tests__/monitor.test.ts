import { describe, it, expect } from "vitest";
import type { MonitorInfo, InputSource, MonitorListResult, AppConfig, SwitchResult } from "../monitor";

describe("Type definitions", () => {
  it("MonitorInfo structure matches expected shape", () => {
    const monitor: MonitorInfo = {
      index: 1,
      model: "Test",
      current_input: 0x0f,
      current_input_name: "DP-1",
      supported_inputs: [{ value: 0x0f, name: "DP-1" }],
      brightness: 50,
      contrast: 70,
      volume: 30,
      power_mode: 1,
    };
    expect(monitor.index).toBe(1);
    expect(monitor.supported_inputs).toHaveLength(1);
  });

  it("MonitorInfo allows null current_input", () => {
    const monitor: MonitorInfo = {
      index: 1,
      model: "Test",
      current_input: null,
      current_input_name: "未知",
      supported_inputs: [],
      brightness: null,
      contrast: null,
      volume: null,
      power_mode: null,
    };
    expect(monitor.current_input).toBeNull();
  });

  it("MonitorListResult with error", () => {
    const result: MonitorListResult = {
      monitors: [],
      error: "检测失败",
    };
    expect(result.error).toBe("检测失败");
    expect(result.monitors).toHaveLength(0);
  });

  it("MonitorListResult without error", () => {
    const result: MonitorListResult = {
      monitors: [
        {
          index: 1,
          model: "M",
          current_input: 0x11,
          current_input_name: "HDMI-1",
          supported_inputs: [],
          brightness: null,
          contrast: null,
          volume: null,
          power_mode: null,
        },
      ],
      error: null,
    };
    expect(result.error).toBeNull();
    expect(result.monitors).toHaveLength(1);
  });

  it("AppConfig with custom names", () => {
    const config: AppConfig = {
      input_names: { "1-15": "MacBook", "1-17": "Ubuntu" },
      monitor_order: [],
      presets: [],
    };
    expect(Object.keys(config.input_names)).toHaveLength(2);
  });

  it("InputSource has value and name", () => {
    const input: InputSource = { value: 0x0f, name: "DP-1" };
    expect(input.value).toBe(15);
    expect(input.name).toBe("DP-1");
  });

  it("SwitchResult success", () => {
    const result: SwitchResult = { status: "success", message: "已切换到 DP-1" };
    expect(result.status).toBe("success");
    expect(result.message).toContain("DP-1");
  });

  it("SwitchResult warning", () => {
    const result: SwitchResult = { status: "warning", message: "目标端口可能无信号" };
    expect(result.status).toBe("warning");
  });
});
