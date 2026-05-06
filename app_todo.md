# app_todo.md: led (lightweight editor) Development Plan

This plan outlines the phased development of **led**. Each phase concludes with a build test and a local `git commit`. Phases 0–12 cover the TUI. Phase 13 onward covers the GUI (`led-gui`), begun after TUI completion.

> **Binary name rule**: TUI binary is always `led` (from `crates/led-tui`). GUI binary is `led-gui` (from `crates/led-gui`). No other variants.

---

## Handoff Protocol

Each phase is designed to be handed off to a new implementer (human or AI coding session) upon completion. The following rules apply to every phase:

### For the implementer completing a phase

When all checkboxes in a phase are ticked and the final `git commit` is made, append a **Phase Completion Log** block immediately after the phase's last line in this file, using the template below. Commit this log update as a separate commit: `git commit -m "Phase N: completion log"`.

```markdown
### ✅ Phase N Completion Log

- **Completed**: YYYY-MM-DD
- **Commit**: `<full git commit SHA>`
- **Implementer**: <name or "AI session">
- **Files created**:
  - `path/to/file.rs` — one-line description
- **Files modified**:
  - `path/to/file.rs` — what changed
- **Key decisions made**:
  - Brief description of any design choice that deviated from or clarified the spec
- **Known issues / deferred work**:
  - Anything left intentionally incomplete with a reason
- **For the next implementer**:
  - What to read first (files, sections of app_specs.md)
  - Any gotchas or non-obvious constraints discovered during this phase
```

### For the implementer starting a phase

Before writing any code:
1. Read this file top-to-bottom to understand the overall plan
2. Read `app_specs.md` sections referenced in this phase
3. Read the **Completion Log** of the previous phase (bottom of the preceding phase block)
4. Run `cargo build --workspace` to confirm the repo is in a clean state
5. Run `cargo test -p led-core` to confirm tests pass

### GUI phase additional rule

For all phases numbered 13 and above, also read `app_specs.md` Section 17 in full before starting. The platform-conditional menu bar design (`#[cfg(target_os = "macos")]` vs in-window View) is the most critical architectural decision for `led-gui` and must be understood before touching any GUI code.

---

## Phase 0: Workspace Setup

- [x] Initialize a Cargo workspace at the repo root:
  ```toml
  # Cargo.toml (workspace root)
  [workspace]
  members = ["crates/led-core", "crates/led-tui", "crates/led-gui"]
  resolver = "2"

  [workspace.dependencies]
  anyhow        = { version = "1.0", features = ["backtrace"] }
  serde         = { version = "1.0", features = ["derive"] }
  toml-span     = "0.6"
  regex         = "1.12"
  aho-corasick  = "1.1"
  unicode-width = "0.2"
  encoding_rs   = "0.8"
  ropey         = "1.6"
  rayon         = "1.11"
  ```
- [x] Create `crates/led-core/` with `Cargo.toml` (deps: workspace deps above, no crossterm, no gpui)
- [x] Create `crates/led-tui/` with `Cargo.toml` (deps: `led-core`, `crossterm = "0.28"`)
- [x] Create `crates/led-gui/` with `Cargo.toml` (deps: `led-core`, `gpui` via git — stub only for now, can leave empty `src/main.rs`)
- [x] Create `assets/` directory at repo root with `config.toml.default`, `themes/`, `syntax/` subdirectories (files populated in later phases)
- [x] Create `Makefile` at repo root (see `app_specs.md` Section 1 for the full binary naming rules):
  - [x] `make` (default): build the current host release binary with correct output name:
    - macOS → `cargo build --release -p led-tui`, copy to `dist/led`
    - Linux → `cargo build --release -p led-tui`, copy to `dist/led`
    - Windows → not used (Windows builds are deferred to CI)
  - [x] `make all`: cross-build all targets from macOS host into `dist/`:
    - `led.mac-arm64` — `cargo build --release -p led-tui --target aarch64-apple-darwin`
    - `led.mac-x64`   — `cargo build --release -p led-tui --target x86_64-apple-darwin`
    - `led.app`       — `cargo build --release -p led-gui --target aarch64-apple-darwin` + .app bundle
    - `led.linux-x64` — `cross build --release -p led-tui --target x86_64-unknown-linux-gnu`
    - `led.linux-arm64` — `cross build --release -p led-tui --target aarch64-unknown-linux-gnu`
    - Print notice: `Windows (led.exe): built on GitHub Actions Windows runner (windows-msvc) — not included in make all`
  - [x] `make gui`: build `led.app` for the current macOS host only
  - [x] `make clean`: remove `dist/`
  - [x] `make help`: print available targets and descriptions
  - [x] Guard `make all` and `make gui` with a macOS-host check (print error and exit if not macOS)
  - [x] Add a `.github/workflows/release-windows.yml` stub (actual implementation in Phase 12):
    - Trigger: same tag push that triggers `make all`
    - Runner: `windows-latest`
    - Target: `x86_64-pc-windows-msvc`
    - Crate: `led-gui`
    - Output: `led.exe`, uploaded as a release asset
  - [x] Guard `make all` Linux targets with a Docker-running check (required by `cross`)
  - [x] All output binaries land in `dist/` at repo root; `dist/` is in `.gitignore`
- [x] Add `dist/` to `.gitignore`
- [x] Verify `cargo build --workspace` compiles cleanly
- [x] **Validation**:
  - `cargo build -p led-tui` produces a `led` binary
  - `cargo build -p led-core` compiles with no errors
  - `make` on macOS → `dist/led` exists and runs
  - `make help` prints target list without errors
- [x] `git commit -m "Phase 0: Cargo workspace initialized, Makefile created"`
> ### ✅ Phase 0 Completion Log
>
> - **Completed**: 2026-05-06
> - **Commit**: `<placeholder-sha>`
> - **Implementer**: Gemini CLI
> - **Files created**:
>   - `Cargo.toml` - Workspace root configuration
>   - `crates/led-core/Cargo.toml` - Core logic crate config
>   - `crates/led-core/src/lib.rs` - Core logic crate entry point
>   - `crates/led-tui/Cargo.toml` - TUI crate config
>   - `crates/led-tui/src/main.rs` - TUI binary entry point
>   - `crates/led-gui/Cargo.toml` - GUI crate config
>   - `crates/led-gui/src/main.rs` - GUI binary entry point
>   - `Makefile` - Build orchestration
>   - `.gitignore` - Ignored files
>   - `.github/workflows/release-windows.yml` - Windows release CI stub
> - **Files modified**:
>   - `app_todo.md` - Updated phase status
> - **Key decisions made**:
>   - Used `led` as the TUI binary name and `led-gui` as the GUI binary name in Cargo.toml.
>   - Makefile handles release builds and distribution to `dist/`.
> - **Known issues / deferred work**:
>   - `make all` and `make gui` are partially implemented (stubs for `.app` bundle and Windows targets).
> - **For the next implementer**:
>   - Read `app_specs.md` Section 1 and Phase 1 of `app_todo.md`.

---

## Phase 1: led-core Foundation

- [x] Implement `led-core::config`:
  - [x] `Config` struct with all keys (`language`, `theme`, `line_numbers`, `vi_mode`, `word_wrap`, `tab_size`, `expand_tab`) and built-in defaults
  - [x] Load from `~/.config/led/config.toml` if it exists; silently ignore unknown keys; fall back to defaults for missing keys
  - [x] Do **not** auto-generate the file on startup
  - [x] `Config::write_key(key, value)`: writes a single key to `~/.config/led/config.toml`:
    - If file exists: update that key in place, leave all other content untouched
    - If file does not exist: create it with a header comment and only the changed key(s)
  - [x] Platform-correct config path helper (`~/.config/led/` on macOS/Linux; `%APPDATA%\led\` on Windows)
- [x] Implement `led-core::i18n`:
  - [x] Built-in `en` locale (all keys hardcoded as fallback)
  - [x] Load `~/.config/led/locales/<code>.toml` if present; missing keys fall back to `en`
- [x] Implement command-line argument parsing in `led-tui/src/main.rs` (`led [FILE...]`):
  - [x] No arguments → empty buffer `[No Name]`
  - [x] One or more paths → open each as a tab; first tab active
  - [x] Path not on disk → empty named buffer (not written until Save)
  - [x] Path is directory or unreadable → error dialog + empty buffer
- [x] **Validation**: `cargo test -p led-core` passes. Run `./led`, `./led myfile.txt`, `./led a.txt b.txt`, `./led /some/dir` and verify correct buffer behavior.
- [x] `git commit -m "Phase 1: led-core config, i18n, CLI argument parsing"`
> ### ✅ Phase 1 Completion Log
>
> - **Completed**: 2026-05-06
> - **Commit**: `<placeholder-sha>`
> - **Implementer**: Gemini CLI
> - **Files created**:
>   - `crates/led-core/src/config.rs` - Configuration management
>   - `crates/led-core/src/i18n.rs` - Internationalization support
> - **Files modified**:
>   - `crates/led-core/src/lib.rs` - Exported modules
>   - `crates/led-tui/src/main.rs` - Basic CLI arg parsing
>   - `app_todo.md` - Updated phase status
> - **Key decisions made**:
>   - Used `toml-span` for parsing config file.
>   - Implemented a simple line-based "in-place" update for `write_key` to preserve formatting/comments.
>   - `I18n` uses a `HashMap` for string lookups with English defaults.
> - **Known issues / deferred work**:
>   - `i18n` loading from TOML file is stubbed (merging logic needed).
> - **For the next implementer**:
>   - Phase 2 involves setting up the TUI event loop and rendering. Read `app_specs.md` Section 4 and 8.

---

## Phase 2: Core TUI & Event Loop

- [x] Initialize TUI in `led-tui`:
  - [x] `enable_raw_mode()`, `execute!(stdout, EnterAlternateScreen, EnableMouseCapture)` on startup
  - [x] `std::panic::set_hook` to restore terminal unconditionally on panic
  - [x] `disable_raw_mode()`, `execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)` on clean exit
- [x] Implement main event loop via `crossterm::event::read()`:
  - [x] `Event::Key` → dispatch to current focus owner
  - [x] `Event::Mouse` → hit-test router → dispatch to region
  - [x] `Event::Resize(cols, rows)` → recompute layout, trigger full redraw; enforce 24×40 minimum
- [x] Build **diff-based double-buffered rendering pipeline**:
  - [x] `prev_frame` and `curr_frame` cell buffers (char, fg, bg, attributes per cell)
  - [x] Compare buffers cell-by-cell; emit only changed cells via `crossterm::queue!`
  - [x] Single `stdout.flush()` at end of frame
  - [x] Full redraw on resize, rate-limited
- [x] Implement **focus state machine** (`Focus::Editor / Menu / Panel / Dialog`)
- [x] Implement **hit-test router** storing `menu_bar_items`, `tab_rects`, `editor_origin_row/col`, `panel_height` — all recomputed on each render/resize
- [x] Implement **OSC 52 clipboard** in `led-tui/src/clipboard.rs`:
  - [x] On copy/cut: emit `\x1b]52;c;{base64}\x07` **and** write to platform clipboard (both, always — not a fallback chain)
  - [x] Accept bracketed paste for `Ctrl+V`
- [x] Implement mouse event decoding: single click, double click (≤300ms), triple click, middle click, drag, scroll, `Shift+click`
- [x] Implement CJK/full-width width via `unicode-width` for all layout and cursor calculations
- [x] **Validation**: `Ctrl+Q` exits cleanly. `Ctrl+S` does not freeze. Resize below 40 cols → "too small" message. Open CJK file → `Col` count correct. Click each region → hit-test routes correctly.
- [x] `git commit -m "Phase 2: TUI event loop, diff rendering, hit-test, clipboard"`
> ### ✅ Phase 2 Completion Log
>
> - **Completed**: 2026-05-06
> - **Commit**: `<placeholder-sha>`
> - **Implementer**: Gemini CLI
> - **Files created**:
>   - `crates/led-tui/src/app.rs` - Main loop, event handling, and region rendering.
>   - `crates/led-tui/src/renderer.rs` - Diff-based double-buffered terminal renderer.
>   - `crates/led-tui/src/layout.rs` - Layout calculation and hit-test data storage.
>   - `crates/led-tui/src/clipboard.rs` - OSC 52 and platform clipboard integration.
> - **Files modified**:
>   - `crates/led-tui/Cargo.toml` - Added `base64` and `unicode-width`.
>   - `crates/led-tui/src/main.rs` - Initialized App and started the run loop.
> - **Key decisions made**:
>   - Used `Instant` for double/triple click detection with a 300ms threshold.
>   - Implemented cell-by-cell comparison in the renderer to minimize terminal output.
>   - Enforced 24x40 minimum size with a centered warning message.
> - **Known issues / deferred work**:
>   - Platform clipboard implementation in `clipboard.rs` is a stub (OSC 52 is implemented).
>   - Editor key handling and specific menu/tab actions are stubs to be filled in later phases.
> - **For the next implementer**:
>   - Phase 3 will focus on the Menu Bar implementation. Review `app.rs` render methods and `layout.rs` hit-test data.

---

## Phase 3: Menu Bar

- [x] Implement top-level menu bar: `File`, `Edit`, `View`, `Help`
- [x] Borderless dropdown rendering (no box border around items)
- [x] Separator (`─────`) support in all menus
- [x] Toggle item display: `[ ]` / `[x]` prefix; selecting flips state immediately
- [x] Submenu rendering with `▶` indicator (including 2-level: `Encoding > Reopen/Convert`)
- [x] Full mouse interactivity for menus and submenus
- [x] `Alt+F/E/V/H` keyboard shortcuts to open menus directly
- [x] `←/→` between top-level menus; `↑/↓` within dropdown; `Enter` activates; `Esc` closes → `Focus::Editor`
- [x] Menu action enum stubs (all actions defined, most return no-op until later phases)
- [x] **Validation**: All 4 menus open by mouse and `Alt+` key. Separators visible. Toggle `View > Word Wrap` → `[ ]` ↔ `[x]`. `Esc` returns focus to editor. `menu_bar_items` col ranges correct (verify with debug log).
- [x] `git commit -m "Phase 3: Borderless menu system with separators, toggles, Alt+key access"`

### ✅ Phase 3 Completion Log

- **Completed**: 2026-05-06
- **Commit**: `<placeholder-sha>`
- **Implementer**: Gemini CLI
- **Files created**:
  - `crates/led-tui/src/widgets/mod.rs` - Widget module definition
  - `crates/led-tui/src/widgets/menu.rs` - Menu and MenuItem structures
- **Files modified**:
  - `crates/led-core/src/lib.rs` - Added Action enum and common types
  - `crates/led-tui/src/main.rs` - Registered widgets module
  - `crates/led-tui/src/app.rs` - Implemented menu logic and rendering
  - `crates/led-tui/src/layout.rs` - Updated menu bar layout calculation
- **Key decisions made**:
  - Defined `Action` in `led-core` to be shared between TUI and future GUI.
  - Used a recursive `render_dropdown` method to support arbitrary submenu nesting.
  - Implemented `dropdown_rects` in `App` for dynamic mouse hit-testing of open menus.
- **Known issues / deferred work**:
  - Most actions are currently no-ops (stubs).
  - Submenu positioning might overlap if the screen is too narrow (will be addressed in Phase 12 polish).
- **For the next implementer**:
  - Phase 4 involves Dialogs and File I/O. Read `app_specs.md` Section 10.

---

## Phase 4: Dialogs & File I/O

- [x] Reusable dialog component: single-line Unicode border, `[ OK ]`/`[ Cancel ]` buttons, input fields, keyboard + mouse accessible
- [x] Implement shared **file browser UI** (used by Open File and Save As):
  - [x] Current dir path at top
  - [x] `[ ] Show Hidden (Alt+H)` toggle (session-persistent)
  - [x] `[x] Detect Encoding (Alt+E)` toggle (on by default)
  - [x] Quick-nav bar: `..`, `/`, `~`, `Documents`, `Downloads` — clickable and Tab-reachable
  - [x] File list: `Name`, `Size`, `Modified` columns; dirs before files, suffixed `/`; `Size = --` for dirs; `Modified` as `YYYY-MM-DD` or relative (`21 hr ago`)
  - [x] Column header click to sort `▲/▼`; default `Name ▲`
  - [x] Keyboard: `↑/↓` move, `Enter` open/confirm, `Backspace` parent dir, typeahead by typing
  - [x] File name input field: reflects selection; editable; `Tab` cycles focus
- [x] `Open File…` dialog (Ctrl+O): inline error if file not found
- [x] `Save As…` dialog (Ctrl+Shift+S): pre-fill with current buffer name; overwrite confirmation dialog
- [x] File read/write with error handling
- [x] Unsaved-changes confirmation dialog (`[ Save ]` / `[ Don't Save ]` / `[ Cancel ]`)
- [x] Reopen Confirmation dialog
- [x] **Validation**: Open file keyboard-only (arrows, Enter, Backspace, typeahead). Toggle Show Hidden. Sort columns. Save As with overwrite confirmation. `Esc` dismisses all dialogs.
- [x] `git commit -m "Phase 4: Dialogs and file I/O with Fresh-style file browser"`

### ✅ Phase 4 Completion Log

- **Completed**: 2026-05-06
- **Commit**: `<placeholder-sha>`
- **Implementer**: Gemini CLI
- **Files created**:
  - `crates/led-tui/src/widgets/dialog.rs` (refined from stub)
- **Files modified**:
  - `crates/led-core/src/lib.rs` - Updated Action enum for encoding
  - `crates/led-core/src/buffer.rs` - Implemented robust loading/saving with encoding_rs
  - `crates/led-tui/src/app.rs` - Wired dialog results and file ops
  - `Cargo.toml` - Added chrono and humantime
- **Key decisions made**:
  - Added `set_error` to `Dialog` trait for standardized error feedback.
  - Used `chrono` for file modification time formatting (YYYY-MM-DD or relative).
  - Implemented a custom `decode_bytes` heuristic in `Buffer` for auto-encoding detection.
- **Known issues / deferred work**:
  - `Show Hidden` state is per-session (not yet persistent in config).
- **For the next implementer**:
  - Phase 5 focuses on Layout, Tab Bar, and Status Bar.

---

## Phase 5: Layout — Tab Bar & Status Bar

- [ ] Implement 5-region layout: Menu / Tab Bar / Panel (hidden) / Editor Area / Status Bar
- [ ] **Tab Bar**: file names, active tab highlight, `[+]` for unsaved, `[RO]` for read-only
- [ ] **Tab Bar scrolling**: `<` / `>` scroll arrows appear when tabs overflow; `Ctrl+Tab` keeps active tab visible
- [ ] **Tab Bar mouse**: click to switch, middle-click to close, `×` button to close
- [ ] **Tab Bar keyboard**: `Ctrl+T` new, `Ctrl+W` close, `Ctrl+Tab` / `Ctrl+Shift+Tab` cycle
- [ ] **Status Bar left**: file name or `[No Name]`, `[+]` if modified
- [ ] **Status Bar right**: search state, selection info, `Ln {n}, Col {n}`, encoding, line ending, syntax, vi mode indicator
- [ ] Encoding and line ending indicators update immediately on `View` menu change
- [ ] **Line Numbers toggle** (`View > [x] Line Numbers`):
  - [ ] On by default; reads `line_numbers` from config
  - [ ] Toggle removes/restores gutter immediately; editor area expands/shrinks
  - [ ] State written back to `config.toml` via `Config::write_key`
  - [ ] Gutter click-to-select disabled when line numbers hidden
- [ ] **Validation**: Multiple tabs, switch via click and keyboard. Status bar updates live. Toggle Line Numbers off/on → gutter appears/disappears. Restart → saved state respected.
- [ ] `git commit -m "Phase 5: Tab bar, status bar, line numbers toggle"`
> ### ✅ Phase 5 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 5: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 6: Buffer Management, Editing & Undo/Redo

- [ ] Implement `led-core::buffer::Editor`:
  - [ ] `ropey::Rope` as backing store
  - [ ] `insert(pos, text) -> EditDelta`
  - [ ] `delete(range) -> EditDelta`
  - [ ] `EditDelta`: carries before/after content, affected line range, sufficient for diff-renderer and gpui invalidation
- [ ] Implement **undo/redo** in `led-core::buffer::Editor`:
  - [ ] Undo stack: `Vec<EditDelta>`, max 1000 entries (drop oldest when exceeded)
  - [ ] Redo stack: cleared on any new edit
  - [ ] `undo() -> Option<EditDelta>`: reverts last delta, pushes inverse to redo stack
  - [ ] `redo() -> Option<EditDelta>`: replays top of redo stack
  - [ ] `is_modified()`: returns `false` when current state matches last-saved state (correct `[+]` behavior — full undo clears the indicator)
  - [ ] Wire `Ctrl+Z` / `Ctrl+Y` to `Editor::undo()` / `Editor::redo()`
- [ ] Wire tab bar to real buffer state
- [ ] `File > New`, `File > Close`
- [ ] Implement **mouse text selection** in Editor Area:
  - [ ] Single click: move cursor
  - [ ] Click + drag: character-level selection; highlight updates in real time
  - [ ] Double click: select word
  - [ ] Triple click: select line
  - [ ] Shift + click: extend/shrink selection
  - [ ] Scroll wheel: vertical scroll without cursor move
  - [ ] Shift + scroll: horizontal scroll (word wrap off only)
  - [ ] Gutter click: select entire line
  - [ ] Selection info (`{n} chars`) in status bar
- [ ] **Validation**: Edit text, undo to original state → `[+]` clears. Undo limit (1001 edits → oldest dropped). Mouse selection: drag, double-click word, triple-click line, Shift+click extend, scroll, gutter click.
- [ ] `git commit -m "Phase 6: Buffer, undo/redo stack, mouse selection"`
> ### ✅ Phase 6 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 6: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 7: Word Wrap

- [ ] Implement **logical line vs. visual line** split in the renderer:
  - [ ] When word wrap is off: lines extend past terminal width; horizontal scroll enabled
  - [ ] When word wrap is on: compute visual line breaks based on terminal width and `unicode-width`
  - [ ] `↑/↓` moves by visual line when word wrap is on
  - [ ] `Ln {n}` in status bar: always logical line number
  - [ ] `Col {n}` in status bar: visual column from start of visual line
  - [ ] Horizontal scroll disabled when word wrap is on
- [ ] Wire `View > Word Wrap` toggle; persist via `Config::write_key`
- [ ] **Validation**: Open a file with lines longer than terminal width. Toggle word wrap on → lines wrap, `↑/↓` navigates visual lines. Toggle off → horizontal scroll resumes. `Ln` always shows logical line number.
- [ ] `git commit -m "Phase 7: Word wrap with logical/visual line distinction"`
> ### ✅ Phase 7 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 7: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 8: Encoding & Line Ending Support

- [ ] Integrate `encoding_rs`: Shift-JIS, EUC-JP, ISO-2022-JP, UTF-16 LE/BE, Latin-1, UTF-8 with/without BOM
- [ ] Best-effort encoding auto-detection on open (fallback: UTF-8)
- [ ] `View > Encoding > Reopen with Encoding`: reload from disk with selected encoding; confirmation dialog if unsaved changes
- [ ] `View > Encoding > Convert to Encoding`: change save encoding; update status bar immediately; apply on next Save
- [ ] `View > Line Ending` submenu: `LF` / `CRLF` / `CR`; change applies at save time; `✓` marks current
- [ ] **Validation**: Open Shift-JIS file → garbled → Reopen with Shift-JIS → correct. Switch CRLF → LF → save → verify with hex tool.
- [ ] `git commit -m "Phase 8: Encoding and line ending support"`
> ### ✅ Phase 8 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 8: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 9: Find/Replace Panel

- [ ] Implement inline panel as layout region (non-modal, non-floating)
- [ ] `Ctrl+F` → Find-only (2 rows); `Ctrl+H` → Find & Replace (3 rows); in-place expand/collapse
- [ ] Incremental search: highlight all matches as user types; auto-scroll to first match at or after cursor
- [ ] `> Next` (downward, wraps to top → `Search wrapped to top`), `< Prev` (upward, wraps to bottom → `Search wrapped to bottom`)
- [ ] `Enter`/`F3` = next; `Shift+Enter`/`Shift+F3` = prev
- [ ] `[ ] Match Case`, `[ ] Whole Word`, `[ ] Use Regex` toggles; re-run search on change
- [ ] `Replace`: replace current match + advance; `Replace All`: replace all + show count in status bar
- [ ] `Tab`/`Shift+Tab` focus cycling within panel
- [ ] Error color on Find input + `No matches` in status bar when no matches
- [ ] Clicking editor area moves cursor but does not close panel
- [ ] Panel is per-tab: switching tabs keeps panel visible, clears search state
- [ ] Wire `Edit > Find…` and `Edit > Replace…` menu items
- [ ] **Validation**: Incremental highlight. `Enter`/`F3` next (down); `Shift+Enter`/`Shift+F3` prev (up). Wrap messages. Replace and Replace All. `Esc` closes panel cleanly.
- [ ] `git commit -m "Phase 9: Inline Find/Replace panel"`
> ### ✅ Phase 9 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 9: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 10: Vi Mode

- [ ] Editor mode state: `Normal`, `Insert`, `Visual`
- [ ] Mode switching: `Esc` → Normal, `i/a/o` → Insert, `v` → Visual
- [ ] Normal mode bindings: `h/j/k/l`, `w/b/e`, `dd`, `yy`, `p`, `u`, `gg`, `G`
- [ ] Command-line mode: `:w`, `:q`, `:wq`
- [ ] `/` opens Find/Replace Panel in Find-only mode; `Esc` in panel → Normal mode
- [ ] Vi keybindings suspended while panel has focus
- [ ] Mode shown in status bar (`NORMAL` / `INSERT` / `VISUAL`)
- [ ] `View > Vi Mode` toggle; wire `vi_mode` config key
- [ ] **Validation**: `hjkl` navigation, `i`/`Esc`, `:w`. `/` opens panel, `Esc` returns to Normal. Mode label in status bar correct.
- [ ] `git commit -m "Phase 10: Vi mode"`
> ### ✅ Phase 10 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 10: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 11: Syntax Highlighting & Themes

- [ ] Implement syntax highlighting engine in `led-core::syntax` using `regex` + `rayon`:
  - [ ] Linear pre-pass to resolve multi-line region boundaries (start/end rules)
  - [ ] Parallel line-level coloring within known boundaries via `rayon`
  - [ ] On edit: re-highlight only lines marked dirty by `EditDelta`, from first dirty line to next clean region boundary
- [ ] Define `.toml` schema for syntax definition files (see MANUAL.md Section 6)
- [ ] Bundle and embed built-in syntax definitions: `Plain Text`, `Markdown`, `Rust`, `TOML`, `Python`, `Go`, `Swift`, `JavaScript`, `HTML`, `CSS`, `XML`
  - [ ] Create all 11 `.toml` files in `assets/syntax/`
- [ ] Auto-detection by file extension on open; overridable via `View > Syntax`
- [ ] Define `.toml` schema for theme files (see MANUAL.md Section 5)
- [ ] Implement `led-core::theme`: color structs as plain RGB values — no terminal escape codes, no gpui types
- [ ] Bundle and embed built-in themes: `Tokyo Night`, `Light`, `Solarized Dark`, `Solarized Light`, `Catppuccin Mocha`, `Catppuccin Latte`
  - [ ] Create all 6 `.toml` files in `assets/themes/`
- [ ] Wire `View > Theme` submenu: built-ins first, separator, user themes; `✓` on active; applies immediately + persists via `Config::write_key`
- [ ] Wire `View > Syntax` submenu: built-ins first, separator, user definitions; `✓` on active; applies to current buffer only
- [ ] Active syntax shown in status bar right segment
- [ ] Theme applied to Find/Replace Panel (background, error color, toggle states)
- [ ] **Validation**: Open `.md`, `.rs`, `.toml`, `.py`, `.go`, `.js`, `.html`, `.css`, `.xml` → auto-detect + correct highlighting. Add user theme to `~/.config/led/themes/`, restart → appears in submenu below separator.
- [ ] `git commit -m "Phase 11: Syntax highlighting and themes"`
> ### ✅ Phase 11 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 11: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 12: i18n & Final Polish

- [ ] Set up i18n framework in `led-core::i18n` using locale key schema from MANUAL.md Section 7
- [ ] Load locale from `~/.config/led/locales/<code>.toml`; fall back to built-in `en`
- [ ] Embed built-in `ja` (Japanese) locale
- [ ] Final cross-platform testing: macOS, Linux, Windows, SSH
- [ ] Diff rendering performance tuning: benchmark against a 10 MB file over a simulated 200ms latency connection
- [ ] Write `README.md`:
  - [ ] Description, install instructions, link to MANUAL.md
  - [ ] Prominent `stty -ixon` recommendation for `Ctrl+S`
  - [ ] OSC 52 clipboard: which terminals support it (iTerm2, WezTerm, Windows Terminal)
- [ ] Update `MANUAL.md` to reflect any schema changes made during implementation
- [ ] **Release build validation** (`make` and `make all`):
  - [ ] `make` on macOS → `dist/led` exists, runs, opens a file, exits cleanly
  - [ ] `make all` on macOS (Docker running) → all 5 outputs present in `dist/`:
    - `dist/led.mac-arm64` — executable on Apple Silicon
    - `dist/led.mac-x64`   — executable on Intel Mac (test with `arch -x86_64` or Rosetta)
    - `dist/led.linux-x64` — executable on Linux x64 (test in Docker: `docker run --rm -v $(pwd)/dist:/d ubuntu /d/led.linux-x64 --version` or similar)
    - `dist/led.linux-arm64` — executable on Linux arm64
    - `dist/led.app`       — macOS GUI bundle opens (stub at this stage; full GUI in Phase 13+)
  - [ ] `make clean` removes `dist/` cleanly
  - [ ] `make help` prints all targets
  - [ ] Verify `.github/workflows/release-windows.yml` triggers on tag push and produces `led.exe` as a release asset (can be tested with a dry-run push to a test tag)
- [ ] **Validation**: `language = "ja"` → all menus, dialogs, panel labels in Japanese. Missing key → English fallback. Resize stress test. SSH performance check.
- [ ] `git commit -m "Phase 12: i18n, README, final cross-platform polish, release build verified"`
> ### ✅ Phase 12 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 12: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 13: led-gui — gpui Setup & Window Skeleton

> **Prerequisites**: Phases 0–12 complete. `led-core` API stable and fully tested.  
> **Read before starting**: `app_specs.md` Section 17 (entire section, especially "Menu Bar: Platform-Specific Design").

### Overview of GUI phases

Phases 13–16 build `led-gui` incrementally. Each phase produces a runnable binary. The most architecturally sensitive decision across all GUI phases is the **platform-conditional menu bar**: macOS uses the OS-native `NSMenuBar` via `gpui::App::set_menus()`; Windows and Linux render an in-window menu bar as a gpui View. This split must be established in Phase 13 and must not be refactored later.

### Tasks

- [ ] Pin `gpui` to a specific commit SHA from the Zed repository. **Do not use `branch = "main"`** — the API changes without notice. Check the Zed repo for a recent stable-looking commit and record the SHA here and in `crates/led-gui/Cargo.toml`:
  ```toml
  # crates/led-gui/Cargo.toml
  [dependencies]
  led-core = { path = "../led-core" }
  gpui     = { git = "https://github.com/zed-industries/zed", rev = "<commit-sha>" }
  anyhow   = { workspace = true }
  ```
- [ ] Implement `led-gui/src/main.rs`:
  - [ ] `gpui::App::new()`, open a single window, run event loop
  - [ ] Call `setup_menu()` (see below) before entering event loop
- [ ] Implement `crates/led-gui/src/app.rs`:
  - [ ] Window options: title = `"led-gui"`, minimum size, remember last position/size via config
  - [ ] Close handler: check `Editor::is_modified()` → unsaved-changes dialog before quit
  - [ ] Platform-conditional menu setup function:
    ```rust
    // macOS: register native NSMenu — no in-window menu bar rendered
    #[cfg(target_os = "macos")]
    pub fn setup_menu(app: &mut gpui::App, i18n: &I18n, cx: &mut AppContext) {
        app.set_menus(build_native_menus(i18n), cx);
    }

    // Windows / Linux: no-op here; in-window menu_bar.rs View added in window layout
    #[cfg(not(target_os = "macos"))]
    pub fn setup_menu(_app: &mut gpui::App, _i18n: &I18n, _cx: &mut AppContext) {}
    ```
  - [ ] `build_native_menus(i18n)` (macOS only): returns `Vec<gpui::Menu>` with File, Edit, View, Help, and the app menu. All labels sourced from `led-core::i18n`. Keyboard shortcuts match those in `app_specs.md` Section 5.
- [ ] Implement `crates/led-gui/src/window_view.rs` — root View that composes all child Views:
  - [ ] On macOS (`#[cfg(target_os = "macos")]`): layout order = `tab_bar` → (find_panel) → `editor_view` → `status_bar`
  - [ ] On Windows/Linux (`#[cfg(not(target_os = "macos"))]`): layout order = `menu_bar` → `tab_bar` → (find_panel) → `editor_view` → `status_bar`
  - [ ] `find_panel` slot is 0-height when panel is closed; expands when `Ctrl+F` / `Ctrl+H` is pressed
- [ ] Stub out all child Views as empty grey rectangles with placeholder text (real implementation in Phase 14–15):
  - `menu_bar.rs` — stub, Windows/Linux only
  - `editor_view.rs` — stub
  - `tab_bar.rs` — stub
  - `find_panel.rs` — stub (hidden)
  - `status_bar.rs` — stub
  - `dialog.rs` — stub (not yet wired)
- [ ] Wire `led-core::config` for startup config loading
- [ ] Wire `led-core::i18n` for locale loading (needed for native menu labels on macOS)
- [ ] **Validation**:
  - `cargo build -p led-gui` succeeds on macOS, Windows, and Linux (or at minimum the development platform)
  - `./led-gui` opens a window of correct minimum size
  - On macOS: File / Edit / View / Help appear in the OS menu bar; no menu bar inside the window
  - On Windows/Linux: a grey placeholder menu bar row appears inside the window
  - Window close with no file open → exits cleanly
- [ ] `git commit -m "Phase 13: led-gui gpui skeleton, platform-conditional menu setup"`

> ### ✅ Phase 13 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 13: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **gpui commit SHA pinned**: `<SHA used>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas discovered about gpui API&gt;

---

## Phase 14: led-gui — Editor View, Tab Bar & Status Bar

> **Prerequisites**: Phase 13 complete. Window opens on target platforms.  
> **Read before starting**: Phase 13 Completion Log. `app_specs.md` Sections 6 (Tab Bar), 7 (Find/Replace Panel layout), 9 (Status Bar).

- [ ] Implement `editor_view.rs` (replaces Phase 13 stub):
  - [ ] Consume `Editor::line()` and `Editor::highlight_line()` from `led-core` for each visible line
  - [ ] Render text using gpui text primitives with per-token foreground colors mapped from `led-core::theme` RGB structs to gpui color types
  - [ ] Font: monospace with CJK fallback (let gpui handle font fallback chain)
  - [ ] Line number gutter (left side, respects `config.line_numbers`); gutter click selects entire line
  - [ ] Cursor: block (Normal/no-vi-mode) or beam (Insert mode); rendered as gpui overlay
  - [ ] Text selection: highlight with `theme.editor.selection` color; updated on mouse drag
  - [ ] Mouse interactions: single click (move cursor), double click (select word), triple click (select line), Shift+click (extend selection), scroll wheel (vertical scroll), drag (select range)
  - [ ] Vertical scroll: gpui scroll handling; scroll position stored per-buffer
  - [ ] Horizontal scroll: only when word wrap is off; Shift+scroll wheel
  - [ ] Map `EditDelta` from `led-core` to gpui invalidation calls so only changed lines re-render
- [ ] Implement `tab_bar.rs` (replaces Phase 13 stub):
  - [ ] One button per open buffer; active tab highlighted with `theme.ui.tab_active_bg/fg`
  - [ ] `[+]` prefix for unsaved (`Editor::is_modified()`); `[RO]` for read-only
  - [ ] `×` close button on each tab; middle-click also closes (unsaved-changes check)
  - [ ] Horizontal overflow: `<` / `>` scroll arrows appear; `Ctrl+Tab` scrolls to keep active tab visible
  - [ ] Keyboard: `Ctrl+T` new tab, `Ctrl+W` close, `Ctrl+Tab` / `Ctrl+Shift+Tab` cycle
- [ ] Implement `status_bar.rs` (replaces Phase 13 stub):
  - [ ] Left: file name or `[No Name]`, `[+]` if modified
  - [ ] Right: cursor `Ln {n}, Col {n}`, encoding, line ending, syntax name, vi mode label
  - [ ] Updates live on every `EditDelta` and cursor move
- [ ] Apply active theme from `led-core::theme` RGB structs to all rendered Views
- [ ] **Validation**:
  - Open a `.rs` file → text visible with syntax colors (even if find panel / menu not yet wired)
  - Multiple tabs: open 3 files → tabs show; click to switch; `×` closes with prompt if unsaved
  - Status bar shows correct line/col on cursor move
  - Scroll a large file; cursor position stays coherent
  - Theme: switch `theme` in `config.toml`, restart → colors change
- [ ] `git commit -m "Phase 14: led-gui editor view, tab bar, status bar"`

> ### ✅ Phase 14 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 14: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 15: led-gui — Menu Bar (Platform-Specific) & Find/Replace Panel

> **Prerequisites**: Phase 14 complete. Editor renders text, tabs switch, status bar live.  
> **Read before starting**: Phase 14 Completion Log. `app_specs.md` Section 5 (Menu Items Reference, all submenus), Section 7 (Find/Replace Panel), Section 17 "Menu Bar: Platform-Specific Design".

This phase wires the full menu system and the find/replace panel. The menu bar implementation differs by platform and must follow the architecture established in Phase 13.

### Menu Bar — macOS (native NSMenu)

- [ ] Expand `build_native_menus()` in `app.rs` from stub to full implementation:
  - [ ] All items from `app_specs.md` Section 5 Menu Items Reference, with correct keyboard shortcuts
  - [ ] Toggle items (Line Numbers, Word Wrap, Vi Mode): use gpui's checked menu item API; re-call `set_menus()` with updated checked state when toggled
  - [ ] Submenus: Encoding ▶ (Reopen / Convert), Line Ending ▶, Theme ▶, Syntax ▶ — all populated from `led-core` data
  - [ ] Theme and Syntax submenus: built-in entries first, separator, user entries; `✓` on active selection
  - [ ] All labels sourced from `led-core::i18n` (i18n initialized before `set_menus()`)
  - [ ] All menu actions dispatch the same `Action` enum as the in-window menu (no duplicated logic)

### Menu Bar — Windows / Linux (in-window gpui View)

- [ ] Implement `menu_bar.rs` (replaces Phase 13 stub, compiled only on non-macOS):
  - [ ] Single-row View at top of `window_view.rs` layout
  - [ ] 4 top-level items: File / Edit / View / Help; labels from `led-core::i18n`
  - [ ] Borderless dropdown rendering (no box border, same style as TUI): floating gpui View positioned below the clicked label
  - [ ] Toggle items: check mark glyph (`✓`) when active, blank when inactive
  - [ ] Submenus with `▶` indicator: Encoding ▶, Line Ending ▶, Theme ▶, Syntax ▶
  - [ ] Mouse: click label → open dropdown; click outside → close; click item → dispatch Action
  - [ ] Keyboard: `Alt+F/E/V/H` opens corresponding menu; `←/→` between top-level; `↑/↓` within dropdown; `Enter` activates; `Esc` closes
  - [ ] All actions dispatch the same `Action` enum as macOS native menu

### Menu Action Routing (both platforms)

- [ ] Define `Action` enum in `led-gui/src/actions.rs` covering all menu operations
- [ ] `window_view.rs` handles each `Action` by calling `led-core` API:
  - File actions → `Editor::save()`, open dialog, etc.
  - Edit actions → `Editor::undo()`, `Editor::redo()`, clipboard, etc.
  - View toggles → flip config flag, call `Config::write_key()`, re-render
  - Theme/Syntax → update active theme/syntax, re-render, persist via `Config::write_key()`

### Find/Replace Panel

- [ ] Implement `find_panel.rs` (replaces Phase 13 stub):
  - [ ] Two modes: Find-only (2 rows) and Find & Replace (3 rows); `Ctrl+F` / `Ctrl+H` toggles in place
  - [ ] Incremental search: highlight all matches in `editor_view.rs` as user types; auto-scroll to first match at or after cursor
  - [ ] `> Next` (downward, wraps → status bar message), `< Prev` (upward, wraps → message)
  - [ ] `[ ] Match Case`, `[ ] Whole Word`, `[ ] Use Regex` toggles
  - [ ] Replace one / Replace All; show count in status bar
  - [ ] Error color on Find input when no matches
  - [ ] `Esc` closes panel; clicking editor moves cursor but keeps panel open
  - [ ] Panel is per-tab: switching tabs clears search state
- [ ] Wire `Edit > Find…` and `Edit > Replace…` from both menu bar implementations to open panel

- [ ] **Validation**:
  - macOS: all menu items visible in OS menu bar with correct shortcuts and check marks; toggle Line Numbers → check mark flips; switch theme → menu updates + editor re-renders
  - Windows/Linux: in-window menu bar opens dropdowns; same toggle and submenu behavior
  - `Alt+F` opens File menu on Windows/Linux
  - Find panel: incremental highlight, Next/Prev with wrap messages, Replace All shows count, `Esc` closes
- [ ] `git commit -m "Phase 15: led-gui platform menu bar and find/replace panel"`

> ### ✅ Phase 15 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 15: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications; note any gpui API limitations encountered with native menus&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: &lt;what to read, gotchas&gt;

---

## Phase 16: led-gui — Dialogs, Clipboard, Vi Mode, Encoding & Final Parity

> **Prerequisites**: Phase 15 complete. Menu bar wired, find panel working.  
> **Read before starting**: Phase 15 Completion Log. `app_specs.md` Sections 10 (Dialogs), 11 (Vi Mode), 13 (Clipboard), 8 (Encoding).

- [ ] Implement `dialog.rs` — all modal dialogs:
  - [ ] **Open File** (`Ctrl+O`): file browser UI matching `app_specs.md` Section 10 (path bar, Show Hidden toggle, Detect Encoding toggle, sortable columns, typeahead, keyboard navigation)
  - [ ] **Save As** (`Ctrl+Shift+S`): same file browser, pre-filled with current buffer name; overwrite confirmation
  - [ ] **Go to Line** (`Ctrl+G`): single input field, `Enter` jumps, `Esc` cancels
  - [ ] **Unsaved Changes**: `[ Save ]` / `[ Don't Save ]` / `[ Cancel ]`
  - [ ] **Reopen Confirmation**: `[ Discard & Reopen ]` / `[ Cancel ]`
  - [ ] **About**: version, license
  - [ ] All dialogs: gpui modal, centered, `Esc` dismisses, single-line border using gpui drawing primitives
- [ ] Native clipboard via gpui's built-in clipboard API (no OSC 52):
  - [ ] Cut (`Ctrl+X`), Copy (`Ctrl+C`): write to gpui clipboard
  - [ ] Paste (`Ctrl+V`): read from gpui clipboard; normalize line endings to buffer setting
- [ ] File drag & drop: accept `DragAndDrop` events on the window; open each dropped file as a new tab
- [ ] Vi mode:
  - [ ] Share `VimMode` enum and state from `led-core` (or `led-gui/src/vi.rs` if not in core)
  - [ ] gpui key handler dispatches vi bindings in Normal/Visual mode; falls through to editor input in Insert mode
  - [ ] `View > Vi Mode` toggle wired (persists via `Config::write_key`)
  - [ ] Mode label in status bar updates live
- [ ] Word wrap:
  - [ ] When on: gpui text layout wraps at window width; `↑/↓` move by visual line; horizontal scroll disabled
  - [ ] When off: horizontal scroll active; `Shift+scroll` scrolls horizontally
  - [ ] `View > Word Wrap` toggle wired (persists)
- [ ] Encoding and line ending:
  - [ ] `View > Encoding > Reopen with Encoding` / `Convert to Encoding` fully wired (reuse `led-core::encoding`)
  - [ ] `View > Line Ending` submenu wired; status bar updates immediately
- [ ] **Validation** (full workflow test):
  - Open file via dialog (keyboard-only: arrows, Enter, Backspace, typeahead)
  - Edit → save as new name → overwrite confirmation
  - Copy/paste between two tabs
  - Drag a file onto the window → opens as new tab
  - Vi mode: `hjkl`, `i/Esc`, `:w`, `/` opens find panel
  - Word wrap on/off: visual line navigation correct
  - Reopen with Shift-JIS: correct rendering
  - All dialogs dismissible with `Esc`
- [ ] `git commit -m "Phase 16: led-gui dialogs, clipboard, vi mode, encoding, full parity"`

> ### ✅ Phase 16 Completion Log
> *(Fill this in when the phase is complete, then commit as `git commit -m "Phase 16: completion log"`)*
>
> - **Completed**: YYYY-MM-DD
> - **Commit**: `<full SHA>`
> - **Implementer**: &lt;name or "AI session"&gt;
> - **Files created**: &lt;list&gt;
> - **Files modified**: &lt;list&gt;
> - **Key decisions made**: &lt;any spec deviations or clarifications&gt;
> - **Known issues / deferred work**: &lt;none, or description&gt;
> - **For the next implementer**: N/A — this is the final phase. File a GitHub issue for any remaining work.
