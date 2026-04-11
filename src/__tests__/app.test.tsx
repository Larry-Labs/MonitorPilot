import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import App from "../App";

const mockInvoke = vi.mocked(invoke);

const mockMonitorListResult = {
  monitors: [
    {
      index: 1,
      model: "Test Monitor",
      current_input: 0x0f,
      current_input_name: "DP-1",
      supported_inputs: [
        { value: 0x0f, name: "DP-1" },
        { value: 0x11, name: "HDMI-1" },
      ],
    },
  ],
  error: null,
};

const mockConfig = {
  input_names: {},
};

beforeEach(() => {
  vi.clearAllMocks();
  mockInvoke.mockImplementation(async (cmd: string) => {
    switch (cmd) {
      case "cmd_get_monitors":
        return mockMonitorListResult;
      case "cmd_get_config":
        return mockConfig;
      case "cmd_save_config":
        return undefined;
      case "cmd_switch_input":
        return { status: "success", message: "已切换到 HDMI-1" };
      default:
        throw new Error(`Unknown command: ${cmd}`);
    }
  });
});

describe("App", () => {
  it("renders app title", async () => {
    render(<App />);
    expect(screen.getByText("MonitorPilot")).toBeInTheDocument();
  });

  it("shows loading skeleton initially", () => {
    render(<App />);
    const skeletons = document.querySelectorAll("[data-slot='skeleton']");
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it("displays monitors after loading", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });
  });

  it("calls cmd_get_monitors on mount", async () => {
    render(<App />);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("cmd_get_monitors");
    });
  });

  it("calls cmd_get_config on mount", async () => {
    render(<App />);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("cmd_get_config");
    });
  });

  it("shows empty state when no monitors detected", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return { monitors: [], error: null };
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("未检测到 DDC/CI 兼容显示器")).toBeInTheDocument();
    });
  });

  it("shows error alert when detection fails", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return { monitors: [], error: "m1ddc not found" };
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/m1ddc not found/)).toBeInTheDocument();
    });
  });

  it("shows retry button in error state", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return { monitors: [], error: "检测失败" };
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("重新检测")).toBeInTheDocument();
    });
  });

  it("switches input and shows toast", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });

    await user.click(screen.getByText("HDMI-1"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("cmd_switch_input", {
        monitorIndex: 1,
        inputValue: 0x11,
      });
    });
  });

  it("shows warning toast for partial success", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return mockMonitorListResult;
      if (cmd === "cmd_get_config") return mockConfig;
      if (cmd === "cmd_switch_input")
        return { status: "warning", message: "已发送切换指令到 HDMI-1，但显示器当前仍为 DP-1（目标端口可能无信号）" };
      return undefined;
    });

    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });

    await user.click(screen.getByText("HDMI-1"));

    await waitFor(() => {
      expect(screen.getByText(/目标端口可能无信号/)).toBeInTheDocument();
    });
  });

  it("shows version in footer", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/MonitorPilot v/)).toBeInTheDocument();
    });
  });
});
