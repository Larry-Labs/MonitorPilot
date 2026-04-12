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
The system SHALL verify the actual input state after sending a switch command on all platforms (macOS via m1ddc, Linux/Windows via ddc-hi get_vcp_feature).

#### Scenario: Successful switch
- **WHEN** a switch command is sent and the monitor's current input matches the target after 600ms
- **THEN** the system SHALL return a `SwitchResult` with `status: "success"`

#### Scenario: Target port no signal (immediate rejection)
- **WHEN** a switch command is sent but the monitor's current input still shows the previous input after 600ms
- **THEN** the system SHALL return a `SwitchResult` with `status: "warning"` indicating the target port may have no signal

#### Scenario: Monitor unreachable after switch
- **WHEN** a switch command is sent but DDC readback returns None after 600ms
- **THEN** the system SHALL attempt rollback to the previous input and return an error message

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
The system SHALL present a professional, flat tray menu without emoji.

#### Scenario: Menu structure
- **WHEN** the user right-clicks the tray icon
- **THEN** the menu SHALL show: title with version and author → separator → monitor submenus → separator → refresh + open main window → separator → visit homepage + report issue → separator → quit

#### Scenario: Title display
- **WHEN** the tray menu is displayed
- **THEN** the title SHALL show "MonitorPilot vX.X.X — Larry Gao" in normal (non-grey) text color

### Requirement: Release build logging
The system SHALL enable logging in release builds at Info level.

#### Scenario: Release mode
- **WHEN** the application runs in release mode
- **THEN** the log plugin SHALL be initialized with `Info` level filter

#### Scenario: Debug mode
- **WHEN** the application runs in debug mode
- **THEN** the log plugin SHALL be initialized with `Debug` level filter

### Requirement: Switch failure auto-rollback
The system SHALL automatically attempt to restore the previous input when a switch causes the monitor to become unreachable.

#### Scenario: Monitor unreachable after switch
- **WHEN** a switch command is sent and the subsequent input verification returns None (monitor unreachable)
- **THEN** the system SHALL automatically send a blind rollback command to restore the previous input value

#### Scenario: Rollback success
- **WHEN** the rollback command succeeds and verification confirms recovery
- **THEN** the system SHALL return an error message explaining what happened: switch caused disconnect, auto-recovered to previous input

#### Scenario: Rollback failure
- **WHEN** the rollback command fails or recovery cannot be confirmed
- **THEN** the system SHALL return an error message indicating DDC/CI communication loss and suggest checking cable connection

### Requirement: Polling state protection
The system SHALL not clear the monitor list during silent polling when monitors temporarily become unreachable.

#### Scenario: Transient detection failure during polling
- **WHEN** a silent poll returns empty monitors but the previous state had monitors
- **THEN** the system SHALL keep the previous monitor state in the UI

#### Scenario: Explicit refresh
- **WHEN** the user clicks "重新检测" or the application performs an initial load
- **THEN** the system SHALL update to the actual state, even if empty

### Requirement: Comprehensive DDC/CI logging
The system SHALL log all DDC/CI operations for debugging.

#### Scenario: Monitor detection
- **WHEN** `get_monitors` executes
- **THEN** the system SHALL log: m1ddc binary path, raw output, skipped built-in displays, detected monitors count

#### Scenario: Input switching
- **WHEN** `switch_input` executes
- **THEN** the system SHALL log: switch request (from/to), command result, verification result, rollback attempts if any

### Requirement: Backend DDC operation mutex
The system SHALL serialize all DDC/CI switch operations at the monitor module level.

#### Scenario: Concurrent switch from frontend and tray
- **WHEN** a switch command is invoked while another is in progress (from any source: frontend IPC, tray menu)
- **THEN** the second command SHALL block until the first completes, preventing DDC bus conflicts
- **AND** the lock SHALL be acquired inside `switch_input()` itself, ensuring all callers are protected

### Requirement: Content Security Policy
The system SHALL enforce a strict CSP in `tauri.conf.json`.

#### Scenario: CSP enforcement
- **WHEN** the application loads frontend content
- **THEN** only `self`, `asset:`, and `https://asset.localhost` sources SHALL be allowed; inline styles (`'unsafe-inline'`) SHALL be permitted for Tailwind compatibility

### Requirement: Conditional ddc-hi dependency
The system SHALL only compile `ddc-hi` crate on Linux and Windows.

#### Scenario: macOS build
- **WHEN** the application is built for macOS
- **THEN** `ddc-hi` SHALL NOT be compiled, reducing build time and binary size

### Requirement: m1ddc path caching
The system SHALL cache the resolved m1ddc binary path using `OnceLock`.

#### Scenario: Multiple invocations
- **WHEN** `find_m1ddc()` is called multiple times during the application lifecycle
- **THEN** the path SHALL be resolved only once and cached for subsequent calls

### Requirement: Tray author information
The system SHALL display author information in the tray menu title.

#### Scenario: Author display
- **WHEN** the user opens the tray menu
- **THEN** the title line SHALL include "Larry Gao" as the author alongside the version
