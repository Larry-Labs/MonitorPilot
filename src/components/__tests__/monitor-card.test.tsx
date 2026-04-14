import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { MonitorCard } from "../monitor-card";
import type { MonitorInfo } from "../../types/monitor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve()),
}));

const mockMonitor: MonitorInfo = {
  index: 1,
  model: "LG ULTRAGEAR",
  current_input: 0x0f,
  current_input_name: "DP-1",
  supported_inputs: [
    { value: 0x0f, name: "DP-1" },
    { value: 0x10, name: "DP-2" },
    { value: 0x11, name: "HDMI-1" },
    { value: 0x12, name: "HDMI-2" },
  ],
  brightness: null,
  contrast: null,
  volume: null,
  power_mode: null,
};

describe("MonitorCard", () => {
  it("renders monitor model and index", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    expect(screen.getByText("LG ULTRAGEAR")).toBeInTheDocument();
    expect(screen.getByText("#1")).toBeInTheDocument();
  });

  it("renders all input source buttons", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    expect(screen.getByText("DP-2")).toBeInTheDocument();
    expect(screen.getByText("HDMI-1")).toBeInTheDocument();
    expect(screen.getByText("HDMI-2")).toBeInTheDocument();
  });

  it("marks active input with '当前' label", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    expect(screen.getByText("当前")).toBeInTheDocument();
  });

  it("shows active input in badge", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const badges = screen.getAllByText("DP-1");
    expect(badges.length).toBeGreaterThanOrEqual(1);
  });

  it("calls onSwitch when clicking inactive input", async () => {
    const onSwitch = vi.fn();
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={onSwitch}
        onRename={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByText("HDMI-1"));
    expect(onSwitch).toHaveBeenCalledWith(1, 0x11);
  });

  it("does NOT call onSwitch when clicking active input", async () => {
    const onSwitch = vi.fn();
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={onSwitch}
        onRename={vi.fn()}
      />,
    );
    const activeButton = screen.getByRole("button", { pressed: true });
    await userEvent.click(activeButton);
    expect(onSwitch).not.toHaveBeenCalled();
  });

  it("uses custom names when provided", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{ "1-15": "MacBook", "1-17": "Ubuntu" }}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const macbookElements = screen.getAllByText("MacBook");
    expect(macbookElements.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Ubuntu")).toBeInTheDocument();
  });

  it("disables non-target buttons during switching", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={"1-17"}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const dp2Button = screen.getByRole("button", { name: /切换到 DP-2/ });
    expect(dp2Button).toBeDisabled();
  });

  it("disables all buttons during switching including active", () => {
    const optimisticMonitor = { ...mockMonitor, current_input: 0x11 };
    render(
      <MonitorCard
        monitor={optimisticMonitor}
        switching={"1-17"}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const activeButton = screen.getByRole("button", { name: /HDMI-1/ });
    expect(activeButton).toBeDisabled();
  });

  it("shows loading state on switching target button", () => {
    const optimisticMonitor = { ...mockMonitor, current_input: 0x11 };
    render(
      <MonitorCard
        monitor={optimisticMonitor}
        switching={"1-17"}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const switchingButton = screen.getByRole("button", { name: /HDMI-1/ });
    expect(switchingButton.textContent).toContain("切换中");
  });

  it("enters edit mode on pencil button click", async () => {
    const user = userEvent.setup();
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const editButtons = screen.getAllByTitle("重命名");
    await user.click(editButtons[0]);
    expect(screen.getByPlaceholderText("DP-1")).toBeInTheDocument();
  });

  it("calls onRename when finishing edit", async () => {
    const onRename = vi.fn();
    const user = userEvent.setup();
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={onRename}
      />,
    );
    const editButtons = screen.getAllByTitle("重命名");
    await user.click(editButtons[0]);
    const input = screen.getByPlaceholderText("DP-1");
    await user.clear(input);
    await user.type(input, "MacBook{Enter}");
    expect(onRename).toHaveBeenCalledWith("1-15", "MacBook");
  });

  it("cancels edit on Escape", async () => {
    const onRename = vi.fn();
    const user = userEvent.setup();
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={onRename}
      />,
    );
    const editButtons = screen.getAllByTitle("重命名");
    await user.click(editButtons[0]);
    await user.keyboard("{Escape}");
    expect(onRename).not.toHaveBeenCalled();
    expect(screen.queryByPlaceholderText("DP-1")).not.toBeInTheDocument();
  });

  it("shows DDC controls when monitor has DDC data", () => {
    const monitorWithDdc = { ...mockMonitor, brightness: 50, contrast: 70 };
    render(
      <MonitorCard
        monitor={monitorWithDdc}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    expect(screen.getByText("亮度")).toBeInTheDocument();
    expect(screen.getByText("对比")).toBeInTheDocument();
  });

  it("hides DDC controls when monitor has no DDC data", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    expect(screen.queryByText("亮度")).not.toBeInTheDocument();
    expect(screen.queryByText("对比")).not.toBeInTheDocument();
    expect(screen.queryByText("音量")).not.toBeInTheDocument();
    expect(screen.queryByText("电源")).not.toBeInTheDocument();
  });

  it("has correct aria attributes on buttons", () => {
    render(
      <MonitorCard
        monitor={mockMonitor}
        switching={null}
        customNames={{}}
        onSwitch={vi.fn()}
        onRename={vi.fn()}
      />,
    );
    const activeBtn = screen.getByRole("button", { pressed: true });
    expect(activeBtn).toHaveAttribute("aria-pressed", "true");
    expect(activeBtn).toHaveAttribute("aria-label", "DP-1（当前输入源）");
  });
});
