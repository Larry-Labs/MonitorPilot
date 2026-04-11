## ADDED Requirements

### Requirement: Loading skeleton
The system SHALL display a skeleton placeholder while monitor data is loading.

#### Scenario: Initial load
- **WHEN** the application starts and monitor data has not yet been fetched
- **THEN** the system SHALL display skeleton card placeholders that match the layout of monitor cards

#### Scenario: Fast load
- **WHEN** monitor data loads very quickly
- **THEN** the skeleton may flash briefly; this is an accepted trade-off to avoid adding artificial delays

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

### Requirement: Toast notification for switching operations
The system SHALL provide transient toast notifications for input switching operations.

#### Scenario: Switch in progress
- **WHEN** the user initiates an input switch
- **THEN** the system SHALL display a toast with a spinner and "正在切换输入源..." message at the bottom of the window

#### Scenario: Switch success
- **WHEN** the input switch completes successfully
- **THEN** the system SHALL display a green success toast for 2.5 seconds

#### Scenario: Switch warning (no signal)
- **WHEN** the switch command succeeds but the monitor's actual input does not match the target
- **THEN** the system SHALL display an amber warning toast for 4 seconds indicating the target port may have no signal

#### Scenario: Switch failure
- **WHEN** the switch command fails
- **THEN** the system SHALL display a red error toast for 4 seconds with the error message

### Requirement: Concurrent switch protection
The system SHALL prevent concurrent DDC/CI switch operations.

#### Scenario: Rapid clicks
- **WHEN** the user clicks a switch button while another switch operation is in progress
- **THEN** the system SHALL block the second click, display a "操作进行中，请稍候..." toast, and not initiate a new DDC/CI command

#### Scenario: Silent refresh after switch
- **WHEN** a switch operation completes
- **THEN** the system SHALL refresh monitor state silently (without showing the loading skeleton)

### Requirement: Hot-plug detection
The system SHALL automatically detect monitor connection/disconnection events.

#### Scenario: Polling interval
- **WHEN** the application window is visible and no switch operation is in progress
- **THEN** the system SHALL poll for monitor changes every 5 seconds

#### Scenario: Window hidden
- **WHEN** the application window is hidden (minimized or backgrounded)
- **THEN** the system SHALL pause polling to reduce resource usage

#### Scenario: Window restored
- **WHEN** the application window becomes visible again
- **THEN** the system SHALL resume polling

### Requirement: Active input indication on buttons
The system SHALL clearly indicate the currently active input source on the button itself.

#### Scenario: Active input button
- **WHEN** an input source button corresponds to the monitor's current input
- **THEN** the button SHALL display a green pulsing dot, the input name, and a "当前" label

#### Scenario: Inactive input button
- **WHEN** an input source button does not correspond to the current input
- **THEN** the button SHALL use outline styling without active indicators

### Requirement: Retry detection button
The system SHALL provide a retry mechanism when monitor detection fails or returns empty.

#### Scenario: Detection error
- **WHEN** monitor detection fails with an error
- **THEN** the error alert SHALL include a "重新检测" button

#### Scenario: Empty state
- **WHEN** no monitors are detected
- **THEN** the empty state display SHALL include a "重新检测" button

### Requirement: No scrollbars in small windows
The system SHALL display content without scrollbars when the window is small.

#### Scenario: Small window
- **WHEN** the application window is resized to a small size
- **THEN** the content SHALL be fully visible without horizontal or vertical scrollbars
- **AND** the main content area SHALL use overflow-y-auto with hidden scrollbar styling

### Requirement: MonitorCard performance optimization
The system SHALL minimize unnecessary re-renders of MonitorCard components.

#### Scenario: Input switch on one monitor
- **WHEN** the user switches input on monitor A while monitor B is also displayed
- **THEN** only monitor A's card SHALL re-render; monitor B SHALL be skipped via `React.memo` shallow comparison

### Requirement: Input button accessibility
The system SHALL provide accessible labels for input source buttons.

#### Scenario: Active input button
- **WHEN** a button represents the currently active input source
- **THEN** the button SHALL have `aria-pressed="true"` and an `aria-label` indicating it is the current source

#### Scenario: Inactive input button
- **WHEN** a button represents a non-active input source
- **THEN** the button SHALL have `aria-pressed="false"` and an `aria-label` indicating "切换到 {name}"

### Requirement: Edit button keyboard accessibility
The system SHALL ensure the edit (rename) button is keyboard-accessible.

#### Scenario: Tab navigation
- **WHEN** the user navigates using Tab key
- **THEN** the edit button SHALL be focusable and visible on focus (via opacity transition, not display:none)

### Requirement: Long name truncation
The system SHALL truncate long monitor model names and input display names.

#### Scenario: Long monitor name
- **WHEN** a monitor model name exceeds the available width
- **THEN** the name SHALL be truncated with ellipsis and a `title` attribute for full text on hover
