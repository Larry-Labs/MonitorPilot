## ADDED Requirements

### Requirement: Loading skeleton
The system SHALL display a skeleton placeholder while monitor data is loading.

#### Scenario: Initial load
- **WHEN** the application starts and monitor data has not yet been fetched
- **THEN** the system SHALL display skeleton card placeholders that match the layout of monitor cards

#### Scenario: Fast load
- **WHEN** monitor data loads in under 200ms
- **THEN** the skeleton SHALL still briefly appear to avoid a jarring flash

### Requirement: Dark mode support
The system SHALL support dark mode following the user's system preference.

#### Scenario: System dark mode active
- **WHEN** the user's operating system is in dark mode
- **THEN** the application SHALL render with the dark color scheme defined in CSS variables

#### Scenario: System light mode active
- **WHEN** the user's operating system is in light mode
- **THEN** the application SHALL render with the light color scheme

### Requirement: Structured empty state
The system SHALL display a structured empty state when no monitors are detected.

#### Scenario: No monitors found
- **WHEN** monitor detection completes and returns zero monitors
- **THEN** the system SHALL display an icon, a descriptive message explaining why no monitors were found, and actionable suggestions (check DDC/CI, check cable connections)

### Requirement: Alert-based error display
The system SHALL display errors using a styled Alert component instead of plain text.

#### Scenario: Error during monitor detection
- **WHEN** an error occurs while fetching monitor data
- **THEN** the system SHALL display a destructive Alert with an error icon, title, and description

### Requirement: Responsive grid layout
The system SHALL use a responsive grid layout for input source buttons.

#### Scenario: Narrow window
- **WHEN** the application window width is less than 400px
- **THEN** the input source buttons SHALL display in a 2-column grid

#### Scenario: Normal window
- **WHEN** the application window width is 400px or more
- **THEN** the input source buttons SHALL display in a 3-column grid

### Requirement: Explicit edit button for input renaming
The system SHALL provide an explicit edit button for renaming input sources, in addition to double-click.

#### Scenario: Edit button click
- **WHEN** the user clicks the edit icon next to an input source button
- **THEN** the system SHALL enter inline edit mode for that input source name

#### Scenario: Keyboard accessibility
- **WHEN** a keyboard user focuses the edit button and presses Enter
- **THEN** the system SHALL enter inline edit mode

### Requirement: Correct HTML language attribute
The system SHALL declare the correct language attribute in the HTML document.

#### Scenario: Chinese interface
- **WHEN** the application renders
- **THEN** the `<html>` element SHALL have `lang="zh-CN"`

### Requirement: Shared type definitions
The system SHALL use a single source of truth for TypeScript type definitions shared between components.

#### Scenario: Type consistency
- **WHEN** `InputSource` or `MonitorInfo` types are used in any component
- **THEN** they SHALL be imported from a shared `src/types/monitor.ts` file
