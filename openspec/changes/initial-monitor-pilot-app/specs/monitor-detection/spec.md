## ADDED Requirements

### Requirement: Detect connected monitors
The system SHALL detect all monitors connected to the host that support DDC/CI protocol.

#### Scenario: Multiple monitors connected
- **WHEN** the application starts or user triggers a refresh
- **THEN** the system SHALL return a list of all DDC/CI-capable monitors with their model name and current input source

#### Scenario: No DDC/CI capable monitor found
- **WHEN** no connected monitor supports DDC/CI
- **THEN** the system SHALL display a clear message explaining that no compatible monitors were found and guide the user to enable DDC/CI in their monitor's OSD menu

### Requirement: Read monitor capabilities
The system SHALL provide a list of supported input sources for each detected monitor.

#### Scenario: Monitor reports supported inputs
- **WHEN** a DDC/CI capable monitor is detected
- **THEN** the system SHALL present a predefined list of common input sources (DP-1, DP-2, HDMI-1, HDMI-2, USB-C, VGA, DVI) and dynamically include the current input value if not already in the list

> **实现说明**：当前未实现 VCP 0x60 能力串解析（需连接显示器后调试）。采用启发式预置列表 + 当前输入值动态并入的策略，对绝大多数场景已足够。

### Requirement: Read current input source
The system SHALL read the current active input source of each detected monitor.

#### Scenario: Successfully read current input
- **WHEN** querying a DDC/CI capable monitor
- **THEN** the system SHALL return the VCP code 0x60 current value and map it to a human-readable input source name

#### Scenario: Monitor is in standby
- **WHEN** the monitor is in standby or powered off
- **THEN** the system SHALL indicate the monitor status as unavailable and skip input source reading
