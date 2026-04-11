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
