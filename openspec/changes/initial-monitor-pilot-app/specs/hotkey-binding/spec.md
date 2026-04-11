## ADDED Requirements

### Requirement: Global hotkey registration
The system SHALL support registering global keyboard shortcuts that work regardless of which application has focus.

#### Scenario: Register a hotkey
- **WHEN** the user configures a keyboard shortcut (e.g., Ctrl+Shift+1) for switching to a specific input source on a specific monitor
- **THEN** the system SHALL register this as a global hotkey at the OS level and the shortcut SHALL trigger input switching from any application

#### Scenario: Hotkey conflict
- **WHEN** the user attempts to register a hotkey that is already in use by another application or the OS
- **THEN** the system SHALL warn the user about the conflict and allow them to choose a different shortcut

#### Scenario: Unregister hotkey
- **WHEN** the user removes a configured hotkey binding
- **THEN** the system SHALL immediately unregister the global hotkey and release it for other applications

### Requirement: Hotkey configuration persistence
The system SHALL persist all hotkey configurations across application restarts.

#### Scenario: Application restart
- **WHEN** the application is restarted
- **THEN** all previously configured hotkeys SHALL be automatically re-registered and functional

#### Scenario: Export/Import config
- **WHEN** the user wants to migrate settings to another machine
- **THEN** the configuration file (JSON) SHALL be self-contained and portable, using human-readable key names (e.g., "Ctrl+Shift+M" not numeric codes)

### Requirement: Default hotkey suggestions
The system SHALL suggest default hotkey combinations for common switching scenarios.

#### Scenario: Two-input setup detected
- **WHEN** a monitor with exactly two input sources is detected
- **THEN** the system SHALL suggest a single toggle hotkey (e.g., Ctrl+Shift+F12) that alternates between the two inputs
