# led Devlog
## 2026-05-06
### Completed Phase 10: Vi Mode
- Implemented `Normal`, `Insert`, and `Visual` modes.
- Added support for mode switching via `i`, `a`, `o`, `v`, and `Esc`.
- Implemented core Vi navigation: `h`, `j`, `k`, `l`, `w`, `b`, `e`, `gg`, and `G`.
- Added editing commands: `dd` (delete line), `yy` (yank line), `p` (paste), `u` (undo), and `x` (delete char).
- Implemented basic command-line mode for `:w`, `:q`, and `:wq`.
- Integrated search (`/`) into Vi mode, opening the Find panel.
- Added a Vi mode indicator to the status bar and wired the `View > Vi Mode` toggle.
- Refactored TUI handlers to resolve borrow checker issues while calling editor actions.
- Verified build and basic functionality across all modes.

### Completed Phase 9: Find/Replace Panel
- Implemented inline Find/Replace panel with support for incremental search.
- Added support for `Match Case`, `Whole Word`, and `Use Regex` flags.
- Implemented `Next`/`Prev` navigation with wrap-around messages in the status bar.
- Added `Replace` and `Replace All` functionality, with occurrence count reporting.
- Made search results and status per-buffer, ensuring correct behavior when switching tabs.
- Integrated search highlighting into both word-wrapped and non-wrapped rendering modes.
- Wired `Edit > Find` and `Edit > Replace` menu items and keyboard shortcuts (`Ctrl+F`, `Ctrl+H`).

### Completed Phase 8: Encoding & Line Ending Support
- Improved encoding auto-detection in `led-core` to support UTF-8 (with/without BOM), UTF-16 LE/BE, Shift-JIS, EUC-JP, ISO-2022-JP, and Latin-1.
- Expanded TUI menu system to include all supported encoding and line ending options.
- Implemented `Reopen with Encoding` (disk reload) and `Convert to Encoding` (session change) actions.
- Added radio-style menu checkmarks (`✓`) for mutually exclusive options like Encoding, Line Ending, and Theme.
- Ensured the menu system and status bar stay synchronized with buffer state changes (tab switching, encoding/line ending updates).
- Verified core encoding detection logic in `led-core`.

### Completed Phase 7: Word Wrap
- Implemented logical vs. visual line splitting for display.
- Added visual line wrapping based on terminal width and `unicode-width`.
- Implemented visual navigation (`↑/↓`) when word wrap is enabled, moving by visual line rather than logical line.
- Updated the status bar to show visual column relative to the current visual line.
- Wired the `View > Word Wrap` toggle and ensured it persists to the config.
- Automatically disabled horizontal scrolling when word wrap is active.
- Verified wrapping and navigation logic with unit tests in `led-core`.

### Completed Phase 6: Buffer Management, Editing & Undo/Redo
- Implemented `Editor` core logic using `ropey` for efficient large file support.
- Implemented robust undo/redo stack with a 1000-entry limit.
- Added support for character-level text selection via mouse (drag, double/triple click, Shift+click) and keyboard.
- Implemented real-time buffer rendering in TUI, handling CJK width, tabs, and scrolling.
- Integrated selection and cursor info into the status bar.
- Verified core logic with unit tests for insertion, deletion, and undo/redo behavior.

### Completed Phase 4: Dialogs & File I/O
...

- Refined `FileBrowser` with `Detect Encoding` toggle, sorting, and relative modification times.
- Implemented robust `Buffer` loading and saving with `encoding_rs` (auto-detection and conversion).
- Enhanced `OpenDialog` and `SaveAsDialog` with inline error messages.
- Implemented `ReopenConfirmationDialog` for safe encoding changes.
- Added overwrite confirmation logic to `SaveAs` operation.
- Integrated `chrono` and `humantime` for UI time formatting.

### Completed Phase 3: Menu Bar
- Implemented top-level menu bar (`File`, `Edit`, `View`, `Help`) with localization support.
- Implemented borderless dropdown rendering for a modern TUI look.
- Added support for separators and toggle indicators.
- Implemented nested submenus with recursive rendering.
- Added mouse hit-testing for dropdowns via `dropdown_rects`.
- Implemented `Alt+F/E/V/H` keyboard shortcuts and arrow key navigation.
- Shared `Action` enum in `led-core` for cross-frontend consistency.
- Resolved borrow checker issues related to nested state access in `App`.
