# led Devlog

## 2026-05-06
### Completed Phase 4: Dialogs & File I/O
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
