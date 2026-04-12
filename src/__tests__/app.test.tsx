import { render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
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

  it("shows error alert with guidance when detection fails and monitors empty", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return { monitors: [], error: "m1ddc not found" };
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/m1ddc not found/)).toBeInTheDocument();
      expect(screen.getByText("未检测到 DDC/CI 兼容显示器")).toBeInTheDocument();
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
      const buttons = screen.getAllByText("重新检测");
      expect(buttons.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("switches input and shows success toast", async () => {
    let switched = false;
    const switchedResult = {
      monitors: [{
        ...mockMonitorListResult.monitors[0],
        current_input: 0x11,
        current_input_name: "HDMI-1",
      }],
      error: null,
    };
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return switched ? switchedResult : mockMonitorListResult;
      if (cmd === "cmd_get_config") return mockConfig;
      if (cmd === "cmd_switch_input") {
        switched = true;
        return { status: "success", message: "已切换到 HDMI-1" };
      }
      return undefined;
    });

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

    await waitFor(() => {
      expect(screen.getByText("已切换到 HDMI-1")).toBeInTheDocument();
    }, { timeout: 5000 });
  });

  it("shows error toast when switch fails", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return mockMonitorListResult;
      if (cmd === "cmd_get_config") return mockConfig;
      if (cmd === "cmd_switch_input") throw new Error("切换失败: DDC 通信中断");
      return undefined;
    });

    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });

    await user.click(screen.getByText("HDMI-1"));

    await waitFor(() => {
      expect(screen.getByText(/切换失败/)).toBeInTheDocument();
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
    }, { timeout: 5000 });
  });

  it("disables other input buttons during switch", async () => {
    const threeInputMonitors = {
      monitors: [{
        ...mockMonitorListResult.monitors[0],
        supported_inputs: [
          { value: 0x0f, name: "DP-1" },
          { value: 0x11, name: "HDMI-1" },
          { value: 0x12, name: "HDMI-2" },
        ],
      }],
      error: null,
    };
    const switchedMonitors = {
      monitors: [{
        ...threeInputMonitors.monitors[0],
        current_input: 0x11,
        current_input_name: "HDMI-1",
      }],
      error: null,
    };

    let resolveSwitch: ((v: unknown) => void) | undefined;
    let switched = false;
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return switched ? switchedMonitors : threeInputMonitors;
      if (cmd === "cmd_get_config") return mockConfig;
      if (cmd === "cmd_switch_input") {
        return new Promise((resolve) => { resolveSwitch = resolve; });
      }
      return undefined;
    });

    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });

    await user.click(screen.getByText("HDMI-1"));

    await waitFor(() => {
      expect(screen.getByText("正在切换输入源...")).toBeInTheDocument();
    });

    const hdmi2Button = screen.getByRole("button", { name: /切换到 HDMI-2/ });
    expect(hdmi2Button).toBeDisabled();

    switched = true;
    resolveSwitch!({ status: "success", message: "已切换到 HDMI-1" });

    await waitFor(() => {
      expect(hdmi2Button).not.toBeDisabled();
    }, { timeout: 5000 });
  });

  it("shows version in footer", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/MonitorPilot v/)).toBeInTheDocument();
    });
  });
});

describe("App — polling", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows warning toast after 3 consecutive poll failures", async () => {
    let callCount = 0;
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") {
        callCount++;
        if (callCount === 1) return mockMonitorListResult;
        throw new Error("poll error");
      }
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });

    render(<App />);
    await act(async () => {});

    expect(screen.getByText("Test Monitor")).toBeInTheDocument();

    for (let i = 0; i < 3; i++) {
      act(() => { vi.advanceTimersByTime(3000); });
      await act(async () => {});
    }

    expect(screen.getByText(/显示器状态同步异常/)).toBeInTheDocument();
  }, 15000);

  it("does not clear monitors when poll returns empty but monitors exist", async () => {
    let callCount = 0;
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") {
        callCount++;
        if (callCount === 1) return mockMonitorListResult;
        return { monitors: [], error: null };
      }
      if (cmd === "cmd_get_config") return mockConfig;
      return undefined;
    });

    render(<App />);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });

    expect(screen.getByText("Test Monitor")).toBeInTheDocument();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(3000);
    });

    expect(screen.getByText("Test Monitor")).toBeInTheDocument();
  });
});

describe("App — rename", () => {
  it("rolls back custom name on save failure", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "cmd_get_monitors") return mockMonitorListResult;
      if (cmd === "cmd_get_config") return { input_names: { "1-15": "MyDP" } };
      if (cmd === "cmd_save_config") throw new Error("保存失败");
      return undefined;
    });

    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Monitor")).toBeInTheDocument();
    });

    const editBtn = screen.getByLabelText("编辑 MyDP 的名称");
    await user.click(editBtn);

    const input = screen.getByRole("textbox");
    await user.clear(input);
    await user.type(input, "NewName");
    await user.keyboard("{Enter}");

    await waitFor(() => {
      expect(screen.getByText(/保存失败/)).toBeInTheDocument();
    });

    await waitFor(() => {
      const matches = screen.getAllByText("MyDP");
      expect(matches.length).toBeGreaterThanOrEqual(1);
    });
  });
});
