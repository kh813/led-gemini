use std::io::{self, Stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute,
    style::Color,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use anyhow::Result;
use unicode_width::UnicodeWidthChar;
use crate::renderer::{Renderer, Cell};
use crate::layout::Layout;
use crate::widgets::menu::{Menu, MenuItem};
use crate::widgets::dialog::{self, Dialog, DialogResult};
use led_core::{Action, Config, I18n, Encoding, LineEnding, buffer::Buffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Editor,
    Menu,
    Panel,
    Dialog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingOp {
    None,
    Open,
    SaveAs,
    Exit,
    Close,
}

pub struct App {
    pub focus: Focus,
    pub running: bool,
    pub width: u16,
    pub height: u16,
    pub renderer: Renderer,
    pub layout: Layout,
    pub last_click_time: Instant,
    pub last_click_pos: (u16, u16),
    pub click_count: u8,
    
    pub config: Config,
    pub i18n: I18n,
    pub menus: Vec<Menu>,
    pub active_menu: Option<usize>,
    pub selected_item: usize,
    pub submenu_stack: Vec<(usize, usize)>, // (menu_idx, item_idx)
    pub dropdown_rects: Vec<(u16, u16, u16, u16, usize)>, // x, y, w, h, item_idx
    pub current_dialog: Option<Box<dyn Dialog>>,
    pub pending_op: PendingOp,
    pub target_encoding: Option<Encoding>,
    pub target_line_ending: Option<LineEnding>,
    pub target_path: Option<PathBuf>,
    
    pub buffers: Vec<Buffer>,
    pub active_buffer: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        
        let config = Config::load();
        let i18n = I18n::load(&config.language);
        let menus = Self::build_menus(&i18n, &config);

        let buffers = vec![Buffer::new()];
        let active_buffer = 0;
        let mut layout = Layout::new(width, height);
        layout.recompute(&menus, &buffers, active_buffer, config.line_numbers);

        Ok(Self {
            focus: Focus::Editor,
            running: true,
            width,
            height,
            renderer: Renderer::new(width, height),
            layout,
            last_click_time: Instant::now(),
            last_click_pos: (0, 0),
            click_count: 0,
            config,
            i18n,
            menus,
            active_menu: None,
            selected_item: 0,
            submenu_stack: Vec::new(),
            dropdown_rects: Vec::new(),
            current_dialog: None,
            pending_op: PendingOp::None,
            target_encoding: None,
            target_line_ending: None,
            target_path: None,
            buffers,
            active_buffer,
        })
    }

    fn build_menus(i18n: &I18n, config: &Config) -> Vec<Menu> {
        vec![
            Menu::new(i18n.get("menu.file"), vec![
                MenuItem::Action { label: i18n.get("menu.file.new").to_string(), action: Action::New, shortcut: Some("Ctrl+N".to_string()) },
                MenuItem::Action { label: i18n.get("menu.file.open").to_string(), action: Action::Open, shortcut: Some("Ctrl+O".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.file.save").to_string(), action: Action::Save, shortcut: Some("Ctrl+S".to_string()) },
                MenuItem::Action { label: i18n.get("menu.file.save_as").to_string(), action: Action::SaveAs, shortcut: Some("Ctrl+Shift+S".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.file.close").to_string(), action: Action::Close, shortcut: Some("Ctrl+W".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.file.exit").to_string(), action: Action::Exit, shortcut: Some("Ctrl+Q".to_string()) },
            ]),
            Menu::new(i18n.get("menu.edit"), vec![
                MenuItem::Action { label: i18n.get("menu.edit.undo").to_string(), action: Action::Undo, shortcut: Some("Ctrl+Z".to_string()) },
                MenuItem::Action { label: i18n.get("menu.edit.redo").to_string(), action: Action::Redo, shortcut: Some("Ctrl+Y".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.edit.cut").to_string(), action: Action::Cut, shortcut: Some("Ctrl+X".to_string()) },
                MenuItem::Action { label: i18n.get("menu.edit.copy").to_string(), action: Action::Copy, shortcut: Some("Ctrl+C".to_string()) },
                MenuItem::Action { label: i18n.get("menu.edit.paste").to_string(), action: Action::Paste, shortcut: Some("Ctrl+V".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.edit.find").to_string(), action: Action::Find, shortcut: Some("Ctrl+F".to_string()) },
                MenuItem::Action { label: i18n.get("menu.edit.replace").to_string(), action: Action::Replace, shortcut: Some("Ctrl+H".to_string()) },
                MenuItem::Separator,
                MenuItem::Action { label: i18n.get("menu.edit.select_all").to_string(), action: Action::SelectAll, shortcut: Some("Ctrl+A".to_string()) },
            ]),
            Menu::new(i18n.get("menu.view"), vec![
                MenuItem::Action { label: i18n.get("menu.view.go_to_line").to_string(), action: Action::GoToLine, shortcut: Some("Ctrl+G".to_string()) },
                MenuItem::Separator,
                MenuItem::Toggle { label: i18n.get("menu.view.line_numbers").to_string(), action: Action::ToggleLineNumbers, checked: config.line_numbers },
                MenuItem::Toggle { label: i18n.get("menu.view.word_wrap").to_string(), action: Action::ToggleWordWrap, checked: config.word_wrap },
                MenuItem::Toggle { label: i18n.get("menu.view.vi_mode").to_string(), action: Action::ToggleViMode, checked: config.vi_mode },
                MenuItem::Separator,
                MenuItem::Submenu { label: "Encoding".to_string(), menu: Menu::new("Encoding", vec![
                    MenuItem::Submenu { label: "Reopen with Encoding".to_string(), menu: Menu::new("Reopen", vec![
                        MenuItem::Action { label: "UTF-8".to_string(), action: Action::ReopenWithEncoding(Encoding::Utf8), shortcut: None },
                        MenuItem::Action { label: "Shift-JIS".to_string(), action: Action::ReopenWithEncoding(Encoding::ShiftJis), shortcut: None },
                    ])},
                    MenuItem::Submenu { label: "Convert to Encoding".to_string(), menu: Menu::new("Convert", vec![
                        MenuItem::Action { label: "UTF-8".to_string(), action: Action::ConvertToEncoding(Encoding::Utf8), shortcut: None },
                        MenuItem::Action { label: "Shift-JIS".to_string(), action: Action::ConvertToEncoding(Encoding::ShiftJis), shortcut: None },
                    ])},
                ])},
                MenuItem::Submenu { label: "Line Ending".to_string(), menu: Menu::new("Line Ending", vec![
                    MenuItem::Action { label: "LF".to_string(), action: Action::SetLineEnding(LineEnding::Lf), shortcut: None },
                    MenuItem::Action { label: "CRLF".to_string(), action: Action::SetLineEnding(LineEnding::Crlf), shortcut: None },
                    MenuItem::Action { label: "CR".to_string(), action: Action::SetLineEnding(LineEnding::Cr), shortcut: None },
                ])},
                MenuItem::Separator,
                MenuItem::Submenu { label: "Theme".to_string(), menu: Menu::new("Theme", vec![
                    MenuItem::Action { label: "Tokyo Night".to_string(), action: Action::SetTheme("tokyo-night".to_string()), shortcut: None },
                    MenuItem::Action { label: "Light".to_string(), action: Action::SetTheme("light".to_string()), shortcut: None },
                ])},
                MenuItem::Submenu { label: "Syntax".to_string(), menu: Menu::new("Syntax", vec![
                    MenuItem::Action { label: "Plain Text".to_string(), action: Action::SetSyntax("plain-text".to_string()), shortcut: None },
                    MenuItem::Action { label: "Rust".to_string(), action: Action::SetSyntax("rust".to_string()), shortcut: None },
                ])},
            ]),
            Menu::new(i18n.get("menu.help"), vec![
                MenuItem::Action { label: i18n.get("menu.help.about").to_string(), action: Action::About, shortcut: None },
            ]),
        ]
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        self.init_terminal(&mut stdout)?;

        while self.running {
            if self.width < 40 || self.height < 24 {
                self.render_too_small(&mut stdout)?;
            } else {
                self.render(&mut stdout)?;
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(w, h) => {
                        self.width = w;
                        self.height = h;
                        self.renderer.resize(w, h);
                        self.layout.width = w;
                        self.layout.height = h;
                        self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                    }
                    _ => {}
                }
            }
        }

        self.cleanup_terminal(&mut stdout)?;
        Ok(())
    }

    fn init_terminal(&self, stdout: &mut Stdout) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            EnterAlternateScreen,
            event::EnableMouseCapture,
            cursor::Hide
        )?;
        
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let mut stdout = io::stdout();
            let _ = terminal::disable_raw_mode();
            let _ = execute!(
                stdout,
                LeaveAlternateScreen,
                event::DisableMouseCapture,
                cursor::Show
            );
            original_hook(panic_info);
        }));

        Ok(())
    }

    fn cleanup_terminal(&self, stdout: &mut Stdout) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            stdout,
            LeaveAlternateScreen,
            event::DisableMouseCapture,
            cursor::Show
        )?;
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Char('q') => { self.running = false; return; }
                KeyCode::Char('n') => { self.perform_action(Action::New); return; }
                KeyCode::Char('o') => { self.perform_action(Action::Open); return; }
                KeyCode::Char('s') => { self.perform_action(Action::Save); return; }
                KeyCode::Char('w') => { self.perform_action(Action::Close); return; }
                KeyCode::Tab => {
                    self.active_buffer = (self.active_buffer + 1) % self.buffers.len();
                    self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                    return;
                }
                _ => {}
            }
        }
        if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) && key.code == KeyCode::Tab {
            self.active_buffer = if self.active_buffer == 0 { self.buffers.len() - 1 } else { self.active_buffer - 1 };
            self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
            return;
        }

        // Handle Alt shortcuts for menus
        if key.modifiers == KeyModifiers::ALT {
            match key.code {
                KeyCode::Char('f') => { self.open_menu(0); return; }
                KeyCode::Char('e') => { self.open_menu(1); return; }
                KeyCode::Char('v') => { self.open_menu(2); return; }
                KeyCode::Char('h') => { self.open_menu(3); return; }
                _ => {}
            }
        }
        
        // Dispatch based on focus
        match self.focus {
            Focus::Menu => self.handle_menu_key(key),
            Focus::Dialog => self.handle_dialog_key(key),
            Focus::Editor => {
                // TODO: editor key handling
            }
            _ => {}
        }
    }

    fn handle_dialog_key(&mut self, key: KeyEvent) {
        if let Some(ref mut dialog) = self.current_dialog {
            let result = dialog.handle_key(key);
            self.handle_dialog_result(result);
        }
    }

    fn handle_dialog_result(&mut self, result: DialogResult<dialog::Action>) {
        match result {
            DialogResult::Ok(action) => {
                match action {
                    dialog::Action::ConfirmPath(path) => {
                        match self.pending_op {
                            PendingOp::Open => {
                                match Buffer::from_file(&path) {
                                    Ok(buffer) => {
                                        self.buffers.push(buffer);
                                        self.active_buffer = self.buffers.len() - 1;
                                        self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                                    }
                                    Err(e) => {
                                        if let Some(ref mut dialog) = self.current_dialog {
                                            dialog.set_error(format!("Error: {}", e));
                                        }
                                        return; // Keep dialog open
                                    }
                                }
                            }
                            PendingOp::SaveAs => {
                                if path.exists() {
                                    self.focus = Focus::Dialog;
                                    self.pending_op = PendingOp::SaveAs; // Keep op
                                    self.target_path = Some(path.clone());
                                    self.current_dialog = Some(Box::new(dialog::MessageDialog::new(
                                        "Confirm Overwrite".to_string(),
                                        format!("File '{}' already exists. Overwrite?", path.file_name().unwrap_or_default().to_string_lossy()),
                                        vec![
                                            ("Overwrite".to_string(), dialog::Action::Save),
                                            ("Cancel".to_string(), dialog::Action::Cancel),
                                        ]
                                    )));
                                    return;
                                }
                                if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                                    if let Err(e) = buffer.save_as(&path) {
                                        if let Some(ref mut dialog) = self.current_dialog {
                                            dialog.set_error(format!("Error: {}", e));
                                        }
                                        return;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    dialog::Action::Confirm => {}
                    dialog::Action::Save => {
                        let op = self.pending_op;
                        self.pending_op = PendingOp::None;
                        match op {
                            PendingOp::Close | PendingOp::Exit => {
                                if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                                    if let Ok(()) = buffer.save() {
                                        if op == PendingOp::Exit {
                                            self.running = false;
                                        } else {
                                            self.buffers.remove(self.active_buffer);
                                            if self.buffers.is_empty() {
                                                self.buffers.push(Buffer::new());
                                            }
                                            self.active_buffer = self.active_buffer.min(self.buffers.len() - 1);
                                            self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                                        }
                                    }
                                }
                            }
                            PendingOp::SaveAs => {
                                if let Some(path) = self.target_path.take() {
                                    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                                        let _ = buffer.save_as(&path);
                                        self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                                    }
                                }
                            }
                            _ => {
                                self.perform_action(Action::Save);
                            }
                        }
                    }
                    dialog::Action::DontSave | dialog::Action::Discard => {
                        let op = self.pending_op;
                        self.pending_op = PendingOp::None;
                        match op {
                            PendingOp::Exit => self.running = false,
                            PendingOp::Close => {
                                self.buffers.remove(self.active_buffer);
                                if self.buffers.is_empty() {
                                    self.buffers.push(Buffer::new());
                                }
                                self.active_buffer = self.active_buffer.min(self.buffers.len() - 1);
                                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                            }
                            PendingOp::None => {
                                // Probably a Reopen or other immediate operation
                                if let Some(enc) = self.target_encoding.take() {
                                    if let Some(buffer) = self.buffers.get(self.active_buffer) {
                                        if let Some(path) = buffer.path.clone() {
                                            if let Ok(new_buffer) = Buffer::from_file(&path) {
                                                let mut b = new_buffer;
                                                b.encoding = enc;
                                                self.buffers[self.active_buffer] = b;
                                                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    dialog::Action::Cancel => {}
                }
                self.current_dialog = None;
                self.pending_op = PendingOp::None;
                self.focus = Focus::Editor;
            }
            DialogResult::Cancel => {
                self.current_dialog = None;
                self.pending_op = PendingOp::None;
                self.focus = Focus::Editor;
            }
            DialogResult::Pending => {}
        }
    }

    fn open_menu(&mut self, idx: usize) {
        self.focus = Focus::Menu;
        self.active_menu = Some(idx);
        self.selected_item = 0;
        self.submenu_stack.clear();
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if !self.submenu_stack.is_empty() {
                    self.submenu_stack.pop();
                } else {
                    self.focus = Focus::Editor;
                    self.active_menu = None;
                }
            }
            KeyCode::Left => {
                if self.submenu_stack.is_empty() {
                    if let Some(idx) = self.active_menu {
                        let next_idx = if idx == 0 { self.menus.len() - 1 } else { idx - 1 };
                        self.open_menu(next_idx);
                    }
                } else {
                    self.submenu_stack.pop();
                }
            }
            KeyCode::Right => {
                if self.submenu_stack.is_empty() {
                    if let Some(idx) = self.active_menu {
                        let next_idx = (idx + 1) % self.menus.len();
                        self.open_menu(next_idx);
                    }
                } else {
                    // Check if current item has a submenu
                    let is_submenu = {
                        let menu = self.get_current_active_menu();
                        matches!(menu.items.get(self.selected_item), Some(MenuItem::Submenu { .. }))
                    };
                    if is_submenu {
                        self.submenu_stack.push((self.active_menu.unwrap(), self.selected_item));
                        self.selected_item = 0;
                    } else {
                        if let Some(idx) = self.active_menu {
                            let next_idx = (idx + 1) % self.menus.len();
                            self.open_menu(next_idx);
                        }
                    }
                }
            }
            KeyCode::Up => {
                let items_len = self.get_current_active_menu().items.len();
                loop {
                    self.selected_item = if self.selected_item == 0 {
                        items_len - 1
                    } else {
                        self.selected_item - 1
                    };
                    let is_separator = {
                        let menu = self.get_current_active_menu();
                        matches!(menu.items[self.selected_item], MenuItem::Separator)
                    };
                    if !is_separator {
                        break;
                    }
                }
            }
            KeyCode::Down => {
                let items_len = self.get_current_active_menu().items.len();
                loop {
                    self.selected_item = (self.selected_item + 1) % items_len;
                    let is_separator = {
                        let menu = self.get_current_active_menu();
                        matches!(menu.items[self.selected_item], MenuItem::Separator)
                    };
                    if !is_separator {
                        break;
                    }
                }
            }
            KeyCode::Enter => {
                let item = self.get_current_active_menu().items[self.selected_item].clone();
                match item {
                    MenuItem::Action { action, .. } | MenuItem::Toggle { action, .. } => {
                        self.perform_action(action);
                        self.focus = Focus::Editor;
                        self.active_menu = None;
                    }
                    MenuItem::Submenu { .. } => {
                        self.submenu_stack.push((self.active_menu.unwrap(), self.selected_item));
                        self.selected_item = 0;
                    }
                    MenuItem::Separator => {}
                }
            }
            _ => {}
        }
    }

    fn get_current_active_menu(&self) -> &Menu {
        let mut menu = &self.menus[self.active_menu.unwrap_or(0)];
        for (_m_idx, i_idx) in &self.submenu_stack {
            if let MenuItem::Submenu { menu: ref sub, .. } = menu.items[*i_idx] {
                menu = sub;
            }
        }
        menu
    }

    fn perform_action(&mut self, action: Action) {
        match action {
            Action::New => {
                self.buffers.push(Buffer::new());
                self.active_buffer = self.buffers.len() - 1;
                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
            }
            Action::Open => {
                self.focus = Focus::Dialog;
                self.pending_op = PendingOp::Open;
                self.current_dialog = Some(Box::new(dialog::OpenDialog::new()));
            }
            Action::Save => {
                let needs_save_as = if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                    if buffer.path.is_some() {
                        let _ = buffer.save();
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };
                if needs_save_as {
                    self.perform_action(Action::SaveAs);
                }
            }
            Action::SaveAs => {
                self.focus = Focus::Dialog;
                self.pending_op = PendingOp::SaveAs;
                let current_path = self.buffers.get(self.active_buffer).and_then(|b| b.path.as_ref());
                self.current_dialog = Some(Box::new(dialog::SaveAsDialog::new(current_path)));
            }
            Action::Close => {
                if let Some(buffer) = self.buffers.get(self.active_buffer) {
                    if buffer.modified {
                        self.focus = Focus::Dialog;
                        self.pending_op = PendingOp::Close;
                        self.current_dialog = Some(Box::new(dialog::MessageDialog::new(
                            self.i18n.get("dialog.unsaved.title").to_string(),
                            self.i18n.get("dialog.unsaved.message").to_string(),
                            vec![
                                (self.i18n.get("dialog.save").to_string(), dialog::Action::Save),
                                (self.i18n.get("dialog.dont_save").to_string(), dialog::Action::DontSave),
                                (self.i18n.get("dialog.cancel").to_string(), dialog::Action::Cancel),
                            ]
                        )));
                        return;
                    }
                }
                self.buffers.remove(self.active_buffer);
                if self.buffers.is_empty() {
                    self.buffers.push(Buffer::new());
                }
                self.active_buffer = self.active_buffer.min(self.buffers.len() - 1);
                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
            }
            Action::Exit => {
                let any_modified = self.buffers.iter().any(|b| b.modified);
                if any_modified {
                    // For exit, we might want to check ALL buffers, but for now just active one is simpler
                    // Spec says "Close/Exit with unsaved changes"
                    if let Some(buffer) = self.buffers.get(self.active_buffer) {
                        if buffer.modified {
                            self.focus = Focus::Dialog;
                            self.pending_op = PendingOp::Exit;
                            self.current_dialog = Some(Box::new(dialog::MessageDialog::new(
                                self.i18n.get("dialog.unsaved.title").to_string(),
                                self.i18n.get("dialog.unsaved.message").to_string(),
                                vec![
                                    (self.i18n.get("dialog.save").to_string(), dialog::Action::Save),
                                    (self.i18n.get("dialog.dont_save").to_string(), dialog::Action::DontSave),
                                    (self.i18n.get("dialog.cancel").to_string(), dialog::Action::Cancel),
                                ]
                            )));
                            return;
                        }
                    }
                }
                self.running = false;
            }
            Action::ToggleLineNumbers => {
                self.config.line_numbers = !self.config.line_numbers;
                let _ = Config::write_key("line_numbers", &self.config.line_numbers.to_string());
                self.menus = Self::build_menus(&self.i18n, &self.config);
                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
            }
            Action::ToggleWordWrap => {
                self.config.word_wrap = !self.config.word_wrap;
                let _ = Config::write_key("word_wrap", &self.config.word_wrap.to_string());
                self.menus = Self::build_menus(&self.i18n, &self.config);
            }
            Action::ToggleViMode => {
                self.config.vi_mode = !self.config.vi_mode;
                let _ = Config::write_key("vi_mode", &self.config.vi_mode.to_string());
                self.menus = Self::build_menus(&self.i18n, &self.config);
            }
            Action::About => {
                self.focus = Focus::Dialog;
                self.current_dialog = Some(Box::new(dialog::AboutDialog::new(&self.i18n)));
            }
            Action::ReopenWithEncoding(enc) => {
                if let Some(buffer) = self.buffers.get(self.active_buffer) {
                    self.target_encoding = Some(enc);
                    if buffer.modified {
                        self.focus = Focus::Dialog;
                        self.pending_op = PendingOp::None;
                        self.current_dialog = Some(Box::new(dialog::ReopenConfirmationDialog::new(&self.i18n)));
                    } else {
                        // Reload immediately
                        if let Some(path) = buffer.path.clone() {
                            if let Ok(new_buffer) = Buffer::from_file(&path) {
                                let mut b = new_buffer;
                                b.encoding = enc;
                                self.buffers[self.active_buffer] = b;
                            }
                        }
                    }
                }
            }
            Action::ConvertToEncoding(enc) => {
                if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                    buffer.encoding = enc;
                    buffer.modified = true;
                }
            }
            Action::SetLineEnding(le) => {
                if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
                    buffer.line_ending = le;
                    buffer.modified = true;
                }
            }
            Action::SetTheme(theme) => {
                let _ = Config::write_key("theme", &theme);
                // TODO: apply theme
            }
            Action::SetSyntax(syntax) => {
                // TODO: apply syntax
            }
            _ => {} // TODO: other actions
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let now = Instant::now();
        let (x, y) = (mouse.column, mouse.row);

        if self.focus == Focus::Dialog {
            if let Some(ref mut dialog) = self.current_dialog {
                let (dw, dh) = dialog.dimensions();
                let dx = (self.width.saturating_sub(dw)) / 2;
                let dy = (self.height.saturating_sub(dh)) / 2;
                let result = dialog.handle_mouse(mouse, dx, dy, dw, dh);
                self.handle_dialog_result(result);
            }
            return;
        }

        if mouse.kind == MouseEventKind::Down(event::MouseButton::Left) || mouse.kind == MouseEventKind::Down(event::MouseButton::Middle) {
            if now.duration_since(self.last_click_time) < Duration::from_millis(300)
                && self.last_click_pos == (x, y)
            {
                self.click_count = (self.click_count % 3) + 1;
            } else {
                self.click_count = 1;
            }
            self.last_click_time = now;
            self.last_click_pos = (x, y);

            let is_middle = mouse.kind == MouseEventKind::Down(event::MouseButton::Middle);

            // Handle Menu interaction (Left click only)
            if !is_middle && self.focus == Focus::Menu {
                for (rx, ry, rw, rh, item_idx) in &self.dropdown_rects {
                    if x >= *rx && x < *rx + *rw && y >= *ry && y < *ry + *rh {
                        self.selected_item = *item_idx;
                        let menu = self.get_current_active_menu();
                        let item = menu.items[self.selected_item].clone();
                        match item {
                            MenuItem::Action { action, .. } | MenuItem::Toggle { action, .. } => {
                                self.perform_action(action);
                                self.focus = Focus::Editor;
                                self.active_menu = None;
                            }
                            MenuItem::Submenu { .. } => {
                                self.submenu_stack.push((self.active_menu.unwrap(), self.selected_item));
                                self.selected_item = 0;
                            }
                            MenuItem::Separator => {}
                        }
                        return;
                    }
                }
            }

            if y == 0 {
                // Menu Bar (Left click only)
                if !is_middle {
                    for (idx, (_label, start, end)) in self.layout.menu_bar_items.iter().enumerate() {
                        if x >= *start && x < *end {
                            self.open_menu(idx);
                            return;
                        }
                    }
                }
                // Clicked menu bar but not an item
                if self.focus == Focus::Menu {
                    self.focus = Focus::Editor;
                    self.active_menu = None;
                }
            } else if y == 1 {
                // Tab Bar
                for (idx, start, end) in &self.layout.tab_rects {
                    if x >= *start && x < *end {
                        if is_middle {
                            self.active_buffer = *idx;
                            self.perform_action(Action::Close);
                        } else {
                            // Check if clicked the '×'
                            if x >= *end - 3 && x < *end - 1 {
                                self.active_buffer = *idx;
                                self.perform_action(Action::Close);
                            } else {
                                self.active_buffer = *idx;
                                self.layout.recompute(&self.menus, &self.buffers, self.active_buffer, self.config.line_numbers);
                            }
                        }
                        return;
                    }
                }
            } else if y == self.height - 1 {
                // Status Bar
            } else {
                // Editor or Panel
                if self.focus == Focus::Menu {
                    self.focus = Focus::Editor;
                    self.active_menu = None;
                } else {
                    let (ex, ey, ew, eh) = self.layout.editor_bounds();
                    if x >= ex && x < ex + ew && y >= ey && y < ey + eh {
                        self.focus = Focus::Editor;
                    }
                }
            }
        }
    }

    fn render(&mut self, stdout: &mut Stdout) -> Result<()> {
        self.renderer.clear();
        
        // Render regions
        self.render_menu();
        self.render_tabs();
        self.render_editor();
        self.render_status();

        // Render dialog if active
        if let Some(ref dialog) = self.current_dialog {
            let (dw, dh) = dialog.dimensions();
            let x = (self.width.saturating_sub(dw)) / 2;
            let y = (self.height.saturating_sub(dh)) / 2;
            dialog.render(&mut self.renderer, x, y, dw, dh);
        }

        self.renderer.present(stdout)?;
        Ok(())
    }

    fn render_menu(&mut self) {
        let (x, y, w, _h) = self.layout.menu_bounds();
        
        // Background
        for dx in 0..w {
            self.renderer.set_cell(x + dx, y, Cell {
                ch: ' ',
                bg: Color::DarkGrey,
                ..Default::default()
            });
        }

        // Menu items
        for (idx, (label, start, end)) in self.layout.menu_bar_items.iter().enumerate() {
            let is_active = self.active_menu == Some(idx);
            let bg = if is_active { Color::White } else { Color::DarkGrey };
            let fg = if is_active { Color::Black } else { Color::White };

            for (i, c) in label.chars().enumerate() {
                self.renderer.set_cell(start + i as u16 + 1, y, Cell {
                    ch: c,
                    bg,
                    fg,
                    ..Default::default()
                });
            }
            // Fill padding
            self.renderer.set_cell(*start, y, Cell { ch: ' ', bg, ..Default::default() });
            self.renderer.set_cell(*end - 1, y, Cell { ch: ' ', bg, ..Default::default() });
        }

        // Render open dropdowns
        self.dropdown_rects.clear();
        if let Some(idx) = self.active_menu {
            let start_x = self.layout.menu_bar_items[idx].1;
            let menu = self.menus[idx].clone();
            self.render_dropdown(start_x, 1, &menu, 0);
        }
    }

    fn render_dropdown(&mut self, x: u16, y: u16, menu: &Menu, depth: usize) {
        let items = &menu.items;
        let mut max_width = items.iter().map(|item| match item {
            MenuItem::Action { label, shortcut, .. } => {
                label.chars().count() + shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0)
            }
            MenuItem::Toggle { label, .. } => label.chars().count() + 4,
            MenuItem::Submenu { label, .. } => label.chars().count() + 4,
            MenuItem::Separator => 5,
        }).max().unwrap_or(10) as u16;
        max_width += 2; // Padding

        let bg = Color::DarkGrey;
        let fg = Color::White;
        let active_bg = Color::White;
        let active_fg = Color::Black;

        let is_current_level = depth == self.submenu_stack.len();
        let selected_at_this_level = if depth < self.submenu_stack.len() {
            Some(self.submenu_stack[depth].1)
        } else if is_current_level {
            Some(self.selected_item)
        } else {
            None
        };

        for (i, item) in items.iter().enumerate() {
            let iy = y + i as u16;
            let is_selected = selected_at_this_level == Some(i);
            let item_bg = if is_selected { active_bg } else { bg };
            let item_fg = if is_selected { active_fg } else { fg };

            // Fill background
            for dx in 0..max_width {
                self.renderer.set_cell(x + dx, iy, Cell {
                    ch: ' ',
                    bg: item_bg,
                    ..Default::default()
                });
            }

            if is_current_level {
                self.dropdown_rects.push((x, iy, max_width, 1, i));
            }

            match item {
                MenuItem::Separator => {
                    for dx in 0..max_width {
                        self.renderer.set_cell(x + dx, iy, Cell {
                            ch: '─',
                            bg: item_bg,
                            fg: item_fg,
                            ..Default::default()
                        });
                    }
                }
                MenuItem::Action { label, shortcut, .. } => {
                    let mut cur_ix = x + 1;
                    for c in label.chars() {
                        self.renderer.set_cell(cur_ix, iy, Cell { ch: c, bg: item_bg, fg: item_fg, ..Default::default() });
                        cur_ix += 1;
                    }
                    if let Some(s) = shortcut {
                        let sx = x + max_width - s.len() as u16 - 1;
                        for (j, c) in s.chars().enumerate() {
                            self.renderer.set_cell(sx + j as u16, iy, Cell { ch: c, bg: item_bg, fg: item_fg, ..Default::default() });
                        }
                    }
                }
                MenuItem::Toggle { label, checked, .. } => {
                    let prefix = if *checked { "[x] " } else { "[ ] " };
                    let mut cur_ix = x + 1;
                    for c in prefix.chars().chain(label.chars()) {
                        self.renderer.set_cell(cur_ix, iy, Cell { ch: c, bg: item_bg, fg: item_fg, ..Default::default() });
                        cur_ix += 1;
                    }
                }
                MenuItem::Submenu { label, menu: sub } => {
                    let mut cur_ix = x + 1;
                    for c in label.chars() {
                        self.renderer.set_cell(cur_ix, iy, Cell { ch: c, bg: item_bg, fg: item_fg, ..Default::default() });
                        cur_ix += 1;
                    }
                    self.renderer.set_cell(x + max_width - 2, iy, Cell { ch: '▶', bg: item_bg, fg: item_fg, ..Default::default() });

                    if selected_at_this_level == Some(i) && depth < self.submenu_stack.len() {
                        let sub_clone = sub.clone();
                        self.render_dropdown(x + max_width, iy, &sub_clone, depth + 1);
                    }
                }
            }
        }
    }

    fn render_tabs(&mut self) {
        let (x, y, w, _h) = self.layout.tab_bounds();
        
        // Background
        for dx in 0..w {
            self.renderer.set_cell(x + dx, y, Cell {
                ch: ' ',
                bg: Color::Reset,
                ..Default::default()
            });
        }

        let total_tabs_width: u16 = self.layout.tab_rects.iter().map(|(_, s, e)| e - s + 1).sum();
        let needs_scroll = total_tabs_width > w;

        let display_w = if needs_scroll { w.saturating_sub(4) } else { w };
        let offset_x = if needs_scroll { 2 } else { 0 };

        if needs_scroll {
            // Render arrows
            self.renderer.set_cell(x, y, Cell { ch: '<', bg: Color::DarkGrey, fg: Color::White, ..Default::default() });
            self.renderer.set_cell(x + 1, y, Cell { ch: ' ', bg: Color::DarkGrey, ..Default::default() });
            self.renderer.set_cell(x + w - 2, y, Cell { ch: ' ', bg: Color::DarkGrey, ..Default::default() });
            self.renderer.set_cell(x + w - 1, y, Cell { ch: '>', bg: Color::DarkGrey, fg: Color::White, ..Default::default() });
        }

        let mut current_tab_x = x + offset_x;
        for (i, buffer) in self.buffers.iter().enumerate() {
            let is_active = i == self.active_buffer;
            let name = buffer.path.as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "[No Name]".to_string());
            let modified = if buffer.modified { "[+] " } else { "" };
            let ro = if buffer.read_only { "[RO] " } else { "" };
            let label = format!(" {}{}{} × ", ro, modified, name);
            let tab_width = label.chars().count() as u16;

            // Basic scrolling: just hide tabs that don't fit for now
            // In a real app we'd use self.layout.tab_scroll
            if current_tab_x + tab_width > x + offset_x + display_w {
                break;
            }

            let bg = if is_active { Color::White } else { Color::DarkGrey };
            let fg = if is_active { Color::Black } else { Color::White };

            for (j, c) in label.chars().enumerate() {
                self.renderer.set_cell(current_tab_x + j as u16, y, Cell {
                    ch: c,
                    bg,
                    fg,
                    bold: is_active,
                    ..Default::default()
                });
            }
            current_tab_x += tab_width + 1;
        }
    }

    fn render_editor(&mut self) {
        let (x, y, w, h) = self.layout.editor_bounds();
        let (gx, gy, gw, gh) = self.layout.gutter_bounds();

        // Render gutter
        if gw > 0 {
            for dy in 0..gh {
                // Background
                for dx in 0..gw {
                    self.renderer.set_cell(gx + dx, gy + dy, Cell {
                        ch: ' ',
                        bg: Color::Reset,
                        ..Default::default()
                    });
                }
                
                // Line number (placeholder for now)
                let line_num = (dy + 1).to_string();
                if dy < h {
                    let num_x = gx + gw - line_num.len() as u16 - 2;
                    for (i, c) in line_num.chars().enumerate() {
                        self.renderer.set_cell(num_x + i as u16, gy + dy, Cell {
                            ch: c,
                            fg: Color::DarkGrey,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Render editor area background
        for dy in 0..h {
            for dx in 0..w {
                self.renderer.set_cell(x + dx, y + dy, Cell {
                    ch: ' ',
                    bg: Color::Reset,
                    ..Default::default()
                });
            }
        }
    }

    fn render_status(&mut self) {
        let (x, y, w, _h) = self.layout.status_bounds();
        let buffer = &self.buffers[self.active_buffer];
        
        // Left segment: file name
        let name = buffer.path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[No Name]".to_string());
        let modified = if buffer.modified { "[+] " } else { "" };
        let left_text = format!(" {}{} ", modified, name);

        // Right segment
        let encoding = format!("{:?}", buffer.encoding);
        let line_ending = format!("{:?}", buffer.line_ending);
        let syntax = "Plain Text"; // TODO
        let vi_mode = if self.config.vi_mode { " NORMAL " } else { "" };
        let right_text = format!(" Ln 1, Col 1  {}  {}  {} {}", encoding, line_ending, syntax, vi_mode);
        
        // Fill background
        for dx in 0..w {
            self.renderer.set_cell(x + dx, y, Cell {
                ch: ' ',
                bg: Color::DarkGrey,
                fg: Color::White,
                ..Default::default()
            });
        }

        // Render left
        let mut cur_x = x;
        for c in left_text.chars() {
            let width = c.width().unwrap_or(0) as u16;
            if cur_x + width <= x + w {
                self.renderer.set_cell(cur_x, y, Cell { ch: c, bg: Color::DarkGrey, fg: Color::White, ..Default::default() });
                cur_x += width;
            }
        }

        // Render right (right-aligned)
        let right_width = right_text.chars().count() as u16;
        let mut cur_x = x + w.saturating_sub(right_width);
        for c in right_text.chars() {
            let width = c.width().unwrap_or(0) as u16;
            if cur_x + width <= x + w {
                self.renderer.set_cell(cur_x, y, Cell { ch: c, bg: Color::DarkGrey, fg: Color::White, ..Default::default() });
                cur_x += width;
            }
        }
    }

    fn render_too_small(&self, stdout: &mut Stdout) -> Result<()> {
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        let msg = format!("Terminal too small ({}x{}). Please resize.", self.width, self.height);
        let x = (self.width.saturating_sub(msg.len() as u16)) / 2;
        let y = self.height / 2;
        execute!(stdout, cursor::MoveTo(x, y))?;
        write!(stdout, "{}", msg)?;
        stdout.flush()?;
        Ok(())
    }
}
