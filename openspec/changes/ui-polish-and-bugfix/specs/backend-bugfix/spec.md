## ADDED Requirements

### Requirement: Tray icon ID consistency
The system SHALL set a consistent tray icon ID so that tray menu refresh works correctly.

#### Scenario: Tray refresh after input switch
- **WHEN** a user switches input source via the tray menu
- **THEN** the system SHALL successfully locate the tray icon by its ID and rebuild the menu with updated state

### Requirement: Frontend config error handling
The system SHALL handle errors when loading configuration from the backend.

#### Scenario: Config load failure
- **WHEN** `cmd_get_config` invocation fails
- **THEN** the system SHALL catch the error, log it, and continue with default empty custom names

### Requirement: Dead code removal
The system SHALL NOT contain unused public methods in the backend.

#### Scenario: ConfigManager cleanup
- **WHEN** the codebase is reviewed
- **THEN** the `update_input_name` method SHALL be removed from `ConfigManager` if no caller exists

### Requirement: MIT LICENSE file
The project SHALL include a LICENSE file at the repository root.

#### Scenario: License present
- **WHEN** a user or automated tool checks the repository
- **THEN** a valid MIT LICENSE file SHALL exist at the repository root matching the declaration in `Cargo.toml`

### Requirement: Lockfile in version control
The project SHALL track `package-lock.json` in Git.

#### Scenario: CI build
- **WHEN** GitHub Actions runs `npm ci`
- **THEN** `package-lock.json` SHALL be present in the repository and compatible with `package.json`

### Requirement: Post-switch input verification
The system SHALL verify the actual input state after sending a switch command on macOS.

#### Scenario: Successful switch
- **WHEN** a switch command is sent and the monitor's current input matches the target after 500ms
- **THEN** the system SHALL return a success message

#### Scenario: Target port no signal
- **WHEN** a switch command is sent but the monitor's current input still shows the previous input after 500ms
- **THEN** the system SHALL return a warning message indicating the target port may have no signal

#### Scenario: Verification unavailable
- **WHEN** a switch command is sent but the current input cannot be read after 500ms
- **THEN** the system SHALL return a message indicating verification was not possible

### Requirement: Built-in display filtering
The system SHALL filter out built-in laptop displays from the monitor list.

#### Scenario: MacBook built-in display
- **WHEN** m1ddc reports a display with "(null)" as the model name
- **THEN** the system SHALL label it as "内置显示器" and exclude it from the detected monitor list

### Requirement: Dynamic input source list
The system SHALL dynamically include the current input value in the supported inputs list.

#### Scenario: Unknown current input code
- **WHEN** a monitor reports a current input VCP code not in the predefined list (DP-1/2, HDMI-1/2)
- **THEN** the system SHALL add the current input to the supported list so it can be displayed and highlighted

### Requirement: m1ddc sidecar bundling
The system SHALL bundle the m1ddc binary as a Tauri sidecar for macOS.

#### Scenario: Runtime binary resolution
- **WHEN** the application starts on macOS
- **THEN** the system SHALL first look for m1ddc next to the executable (sidecar), falling back to system PATH

### Requirement: Tray menu formalization
The system SHALL present a professional tray menu without emoji.

#### Scenario: Menu structure
- **WHEN** the user right-clicks the tray icon
- **THEN** the menu SHALL show: version title (disabled) → separator → monitor submenus → separator → refresh + open main window → separator → help submenu → separator → quit

#### Scenario: Help submenu
- **WHEN** the user opens the help submenu
- **THEN** it SHALL contain: about (disabled), separator, visit homepage (opens browser), report issue (opens browser)

### Requirement: Release build logging
The system SHALL enable logging in release builds at Info level.

#### Scenario: Release mode
- **WHEN** the application runs in release mode
- **THEN** the log plugin SHALL be initialized with `Info` level filter

#### Scenario: Debug mode
- **WHEN** the application runs in debug mode
- **THEN** the log plugin SHALL be initialized with `Debug` level filter
