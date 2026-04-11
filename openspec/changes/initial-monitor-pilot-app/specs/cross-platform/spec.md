## ADDED Requirements

### Requirement: macOS support
The application SHALL run on macOS 12 (Monterey) and later, on both Apple Silicon and Intel architectures.

#### Scenario: macOS DDC/CI access
- **WHEN** running on macOS
- **THEN** the system SHALL access DDC/CI via I/O Kit (IODisplayConnect) without requiring any external CLI tools or additional driver installation

#### Scenario: macOS menu bar integration
- **WHEN** running on macOS
- **THEN** the tray icon SHALL appear in the macOS menu bar following platform conventions (right side of menu bar)

### Requirement: Linux support
The application SHALL run on Linux distributions with X11 or Wayland display servers.

#### Scenario: Linux DDC/CI access
- **WHEN** running on Linux
- **THEN** the system SHALL access DDC/CI via i2c-dev kernel module and SHALL detect if the module is loaded and the user has appropriate permissions (i2c group membership)

#### Scenario: Linux permission setup
- **WHEN** the i2c-dev module is not loaded or user lacks permissions
- **THEN** the system SHALL display setup instructions and optionally offer to run the setup commands (with user confirmation)

### Requirement: Windows support
The application SHALL run on Windows 10 (1903) and later.

#### Scenario: Windows DDC/CI access
- **WHEN** running on Windows
- **THEN** the system SHALL access DDC/CI via Win32 Monitor Configuration API (SetVCPFeature / GetVCPFeature) without requiring any external tools or drivers

### Requirement: Consistent user experience
The application SHALL provide a consistent user experience across all supported platforms while respecting platform-specific conventions.

#### Scenario: UI consistency
- **WHEN** the user opens the settings window on any platform
- **THEN** the layout, functionality, and workflow SHALL be identical, with only platform-native UI chrome differences (window title bar, scrollbar style)

#### Scenario: Configuration portability
- **WHEN** the user copies their configuration file from one platform to another
- **THEN** the configuration SHALL be compatible and functional (hotkey modifiers mapped to platform equivalents: Cmd↔Ctrl)
