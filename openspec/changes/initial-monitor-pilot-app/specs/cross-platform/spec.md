## ADDED Requirements

### Requirement: macOS support
The application SHALL run on macOS 12 (Monterey) and later, on both Apple Silicon and Intel architectures.

#### Scenario: macOS DDC/CI access
- **WHEN** running on macOS
- **THEN** the system SHALL access DDC/CI via bundled m1ddc sidecar binary (using Apple IOAVService API), falling back to system PATH if sidecar is not found

#### Scenario: macOS menu bar integration
- **WHEN** running on macOS
- **THEN** the tray icon SHALL appear in the macOS menu bar following platform conventions (right side of menu bar)

### Requirement: Linux support
The application SHALL run on Linux distributions with X11 or Wayland display servers.

#### Scenario: Linux DDC/CI access
- **WHEN** running on Linux
- **THEN** the system SHALL access DDC/CI via i2c-dev kernel module using the `ddc-hi` crate

#### Scenario: Linux permission setup (Future)
- **WHEN** the i2c-dev module is not loaded or user lacks permissions
- **THEN** the user SHALL follow README instructions to load the module and add user to i2c group

> **实现说明**：应用内自动检测 i2c 权限并引导安装尚未实现（需 Linux 环境测试）。当前通过 README 文档说明手动配置步骤。

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
- **THEN** the configuration SHALL be compatible and functional (input custom names are platform-independent)
