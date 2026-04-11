## ADDED Requirements

### Requirement: Detect connected monitors
The system SHALL detect all monitors connected to the host that support DDC/CI protocol.

#### Scenario: Multiple monitors connected
- **WHEN** the application starts or user triggers a refresh
- **THEN** the system SHALL return a list of all DDC/CI-capable monitors with their model name, manufacturer, and current input source

#### Scenario: No DDC/CI capable monitor found
- **WHEN** no connected monitor supports DDC/CI
- **THEN** the system SHALL display a clear message explaining that no compatible monitors were found and guide the user to enable DDC/CI in their monitor's OSD menu

### Requirement: Read monitor capabilities
The system SHALL read each detected monitor's supported input sources via DDC/CI VCP capabilities string.

#### Scenario: Monitor reports supported inputs
- **WHEN** a DDC/CI capable monitor is detected
- **THEN** the system SHALL parse VCP code 0x60 capabilities to enumerate all supported input source types (e.g., DP-1, HDMI-1, USB-C) and their corresponding VCP values

#### Scenario: Capabilities query fails
- **WHEN** the DDC/CI capabilities query fails for a monitor
- **THEN** the system SHALL fall back to common input source codes and allow the user to manually configure the available inputs

### Requirement: Read current input source
The system SHALL read the current active input source of each detected monitor.

#### Scenario: Successfully read current input
- **WHEN** querying a DDC/CI capable monitor
- **THEN** the system SHALL return the VCP code 0x60 current value and map it to a human-readable input source name

#### Scenario: Monitor is in standby
- **WHEN** the monitor is in standby or powered off
- **THEN** the system SHALL indicate the monitor status as unavailable and skip input source reading
