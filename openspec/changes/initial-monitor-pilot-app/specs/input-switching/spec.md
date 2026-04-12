## ADDED Requirements

### Requirement: Switch monitor input source
The system SHALL switch a monitor's input source by writing to VCP code 0x60 via DDC/CI protocol.

#### Scenario: Successful input switch
- **WHEN** the user selects a target input source for a specific monitor
- **THEN** the system SHALL write the corresponding VCP value to code 0x60 and verify the switch (all platforms: 2-round verification at ~0.6s and ~2.0s)

#### Scenario: Switch to already active input
- **WHEN** the user requests switching to the input source that is already active
- **THEN** the system SHALL prevent the switch action (button disabled/inactive, tray menu item disabled) without initiating a DDC/CI write

#### Scenario: Switch command fails
- **WHEN** the DDC/CI write command fails (e.g., communication error, permission denied)
- **THEN** the system SHALL display an error message with the failure reason and suggest troubleshooting steps

### Requirement: Post-switch verification (backend)
The system SHALL verify the actual monitor input after each switch command via DDC readback.

#### Scenario: DDC confirms target input
- **WHEN** the DDC readback matches the target input value after all verification rounds
- **THEN** the system SHALL return `status: "success"`

#### Scenario: DDC confirms different input (target port rejected)
- **WHEN** the DDC readback at any verification round returns a value different from the target
- **THEN** the system SHALL return `status: "warning"` with a message indicating the target port may have no signal, and report the actual current input

#### Scenario: DDC unreachable after switch (monitor lost)
- **WHEN** the DDC readback returns None (monitor unreachable)
- **THEN** the system SHALL attempt automatic rollback to the previous input value, report the outcome, and return an error

### Requirement: Optimistic UI update (frontend)
The system SHALL update the UI immediately after the backend returns a successful switch result, without waiting for polling.

#### Scenario: Backend returns success
- **WHEN** the backend returns `status: "success"`
- **THEN** the frontend SHALL immediately set `current_input` to the target value (optimistic update) and show a success toast

#### Scenario: Backend returns warning
- **WHEN** the backend returns `status: "warning"`
- **THEN** the frontend SHALL NOT apply an optimistic update and SHALL show the warning message as-is

#### Scenario: Backend returns error
- **WHEN** the backend returns an error
- **THEN** the frontend SHALL NOT apply an optimistic update and SHALL show the error message

### Requirement: Post-switch silent UI correction (frontend)
The system SHALL silently correct the UI when a monitor automatically reverts from a successfully-switched input source (indicating the target port has no signal).

#### Scenario: Monitor auto-reverts after successful switch
- **WHEN** a switch was reported as successful, BUT a subsequent poll detects the monitor's `current_input` has changed away from the target
- **THEN** the frontend SHALL silently update the UI button states to reflect the actual input, WITHOUT showing an additional warning toast

#### Scenario: DDC/CI limitation — no-signal port accepted by firmware
- **GIVEN** some monitors accept DDC switch commands to ports that physically exist but have no cable connected (e.g., HDMI-2 with no cable) while rejecting commands to ports that don't exist (e.g., DP-2 on a single-DP monitor)
- **WHEN** a switch to such a port is reported as "success" by the backend
- **THEN** the frontend's polling cycle SHALL detect the monitor's eventual revert and silently update the UI, providing a consistent experience where the button state self-corrects within one polling interval (~3 seconds)

#### Scenario: Monitor firmware determines revert target
- **GIVEN** when a monitor auto-reverts from a no-signal port, its firmware decides the revert target (typically the first port with an active signal, e.g., DP-1), which may differ from the user's previous input
- **WHEN** the polling detects the actual input after a revert
- **THEN** the frontend SHALL display the actual input reported by DDC, and SHALL NOT send additional DDC commands to switch to the previous input (to avoid screen flashing)

### Requirement: Custom input source naming
The system SHALL allow users to assign custom names to input sources for each monitor.

#### Scenario: User renames an input
- **WHEN** the user assigns a custom name (e.g., "MacBook") to a monitor's DP-1 input
- **THEN** the custom name SHALL be displayed everywhere in the UI instead of the technical name and SHALL persist across application restarts

#### Scenario: Reset custom name
- **WHEN** the user clears a custom name
- **THEN** the system SHALL revert to displaying the default technical input source name
