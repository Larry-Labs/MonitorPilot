## ADDED Requirements

### Requirement: Switch monitor input source
The system SHALL switch a monitor's input source by writing to VCP code 0x60 via DDC/CI protocol.

#### Scenario: Successful input switch
- **WHEN** the user selects a target input source for a specific monitor
- **THEN** the system SHALL write the corresponding VCP value to code 0x60 and the monitor SHALL switch to the specified input within 3 seconds

#### Scenario: Switch to already active input
- **WHEN** the user requests switching to the input source that is already active
- **THEN** the system SHALL skip the DDC/CI write and display a notification that the input is already active

#### Scenario: Switch command fails
- **WHEN** the DDC/CI write command fails (e.g., communication error, permission denied)
- **THEN** the system SHALL display an error message with the failure reason and suggest troubleshooting steps

### Requirement: Custom input source naming
The system SHALL allow users to assign custom names to input sources for each monitor.

#### Scenario: User renames an input
- **WHEN** the user assigns a custom name (e.g., "MacBook") to a monitor's DP-1 input
- **THEN** the custom name SHALL be displayed everywhere in the UI instead of the technical name and SHALL persist across application restarts

#### Scenario: Reset custom name
- **WHEN** the user clears a custom name
- **THEN** the system SHALL revert to displaying the default technical input source name
