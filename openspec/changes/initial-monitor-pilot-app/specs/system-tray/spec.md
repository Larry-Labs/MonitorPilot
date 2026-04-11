## ADDED Requirements

### Requirement: System tray presence
The application SHALL run as a system tray (menu bar on macOS) application with a persistent icon.

#### Scenario: Application launched
- **WHEN** the application starts
- **THEN** a tray icon SHALL appear in the system tray / macOS menu bar and the main window SHALL remain hidden by default

#### Scenario: Click tray icon
- **WHEN** the user clicks the tray icon
- **THEN** the system SHALL display a popup menu showing all detected monitors and their current input sources with quick-switch options

### Requirement: Tray quick-switch menu
The tray menu SHALL provide one-click input source switching for each detected monitor.

#### Scenario: Single monitor connected
- **WHEN** only one DDC/CI monitor is detected
- **THEN** the tray menu SHALL list the monitor name followed by all available input sources, with the current one marked as active

#### Scenario: Multiple monitors connected
- **WHEN** multiple DDC/CI monitors are detected
- **THEN** the tray menu SHALL group input sources under each monitor name as a submenu

#### Scenario: Switch from tray menu
- **WHEN** the user clicks an input source option in the tray menu
- **THEN** the system SHALL execute the input switch and update the tray menu to reflect the new active input

### Requirement: Tray context menu
The tray menu SHALL include utility options for application management.

#### Scenario: Access settings
- **WHEN** the user selects "Settings" from the tray menu
- **THEN** the main settings window SHALL open

#### Scenario: Quit application
- **WHEN** the user selects "Quit" from the tray menu
- **THEN** the application SHALL exit and remove the tray icon
