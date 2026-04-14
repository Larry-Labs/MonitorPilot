import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { DdcControls } from "../ddc-controls";
import type { MonitorInfo } from "../../types/monitor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve()),
}));

const baseMonitor: MonitorInfo = {
  index: 1,
  model: "Test",
  current_input: 0x0f,
  current_input_name: "DP-1",
  supported_inputs: [{ value: 0x0f, name: "DP-1" }],
  brightness: null,
  contrast: null,
  volume: null,
  power_mode: null,
};

describe("DdcControls", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders nothing when all values are null", () => {
    const { container } = render(<DdcControls monitor={baseMonitor} />);
    expect(container.innerHTML).toBe("");
  });

  it("renders brightness slider when available", () => {
    const monitor = { ...baseMonitor, brightness: 50 };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByText("亮度")).toBeInTheDocument();
    expect(screen.getByText("50")).toBeInTheDocument();
  });

  it("renders contrast slider when available", () => {
    const monitor = { ...baseMonitor, contrast: 70 };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByText("对比")).toBeInTheDocument();
    expect(screen.getByText("70")).toBeInTheDocument();
  });

  it("renders volume slider when available", () => {
    const monitor = { ...baseMonitor, volume: 30 };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByText("音量")).toBeInTheDocument();
    expect(screen.getByText("30")).toBeInTheDocument();
  });

  it("renders power button when available - ON state", () => {
    const monitor = { ...baseMonitor, power_mode: 1 };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByText("电源")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "开启" })).toBeInTheDocument();
  });

  it("renders power button when available - standby state", () => {
    const monitor = { ...baseMonitor, power_mode: 4 };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByRole("button", { name: "待机" })).toBeInTheDocument();
  });

  it("hides unsupported controls selectively", () => {
    const monitor = { ...baseMonitor, brightness: 50, volume: null, contrast: null, power_mode: null };
    render(<DdcControls monitor={monitor} />);
    expect(screen.getByText("亮度")).toBeInTheDocument();
    expect(screen.queryByText("对比")).not.toBeInTheDocument();
    expect(screen.queryByText("音量")).not.toBeInTheDocument();
    expect(screen.queryByText("电源")).not.toBeInTheDocument();
  });

  it("renders slider with correct initial value", () => {
    const monitor = { ...baseMonitor, brightness: 50 };
    render(<DdcControls monitor={monitor} />);

    const hiddenInput = document.querySelector("input[type='range']") as HTMLInputElement;
    expect(hiddenInput).toBeTruthy();
    expect(hiddenInput.value).toBe("50");
  });

  it("calls power mode toggle on button click", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();

    const monitor = { ...baseMonitor, power_mode: 1 };
    render(<DdcControls monitor={monitor} />);

    await userEvent.click(screen.getByRole("button", { name: "开启" }));

    expect(invoke).toHaveBeenCalledWith("cmd_set_power_mode", {
      monitorIndex: 1,
      mode: 4,
    });
  });
});
