use crate::renderer::{Renderer, Cell};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use crossterm::style::Color;
use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Local};

pub enum DialogResult<T> {
    Ok(T),
    Cancel,
    Pending,
}

pub trait Dialog {
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u16, u16); // width, height
    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16);
    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action>;
    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action>;
    fn set_error(&mut self, _msg: String) {}
}

#[derive(Debug, Clone)]
pub enum Action {
    ConfirmPath(PathBuf),
    Confirm,
    Save,
    DontSave,
    Discard,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_idx: usize,
    pub show_hidden: bool,
    pub detect_encoding: bool,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub input_text: String,
    pub input_focused: bool,
}

impl FileBrowser {
    pub fn new(path: PathBuf) -> Self {
        let mut browser = Self {
            current_dir: path,
            entries: Vec::new(),
            selected_idx: 0,
            show_hidden: false,
            detect_encoding: true,
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
            input_text: String::new(),
            input_focused: false,
        };
        browser.refresh();
        browser
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        
        // Add parent dir if not at root
        if let Some(_) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                is_dir: true,
                size: 0,
                modified: None,
            });
        }

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }
                let metadata = entry.metadata().ok();
                self.entries.push(FileEntry {
                    name,
                    is_dir: entry.file_type().map(|t| t.is_dir()).unwrap_or(false),
                    size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
                    modified: metadata.and_then(|m| m.modified().ok()),
                });
            }
        }

        self.sort();
        self.selected_idx = self.selected_idx.min(self.entries.len().saturating_sub(1));
    }

    pub fn sort(&mut self) {
        self.entries.sort_by(|a, b| {
            // Dirs always before files
            if a.name == ".." { return std::cmp::Ordering::Less; }
            if b.name == ".." { return std::cmp::Ordering::Greater; }
            if a.is_dir != b.is_dir {
                return b.is_dir.cmp(&a.is_dir);
            }

            let res = match self.sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Modified => a.modified.cmp(&b.modified),
            };

            if self.sort_order == SortOrder::Descending {
                res.reverse()
            } else {
                res
            }
        });
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PathBuf> {
        if self.input_focused {
            match key.code {
                KeyCode::Char(c) => {
                    self.input_text.push(c);
                    return None;
                }
                KeyCode::Backspace => {
                    self.input_text.pop();
                    return None;
                }
                KeyCode::Enter => {
                    if !self.input_text.is_empty() {
                        return Some(self.current_dir.join(&self.input_text));
                    }
                }
                KeyCode::Tab => {
                    self.input_focused = false;
                    return None;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    if let Some(entry) = self.entries.get(self.selected_idx) {
                        if !entry.is_dir {
                            self.input_text = entry.name.clone();
                        }
                    }
                }
                None
            }
            KeyCode::Down => {
                if self.selected_idx + 1 < self.entries.len() {
                    self.selected_idx += 1;
                    if let Some(entry) = self.entries.get(self.selected_idx) {
                        if !entry.is_dir {
                            self.input_text = entry.name.clone();
                        }
                    }
                }
                None
            }
            KeyCode::Enter => {
                if let Some(entry) = self.entries.get(self.selected_idx) {
                    let path = self.current_dir.join(&entry.name);
                    if entry.is_dir {
                        if entry.name == ".." {
                            if let Some(parent) = self.current_dir.parent() {
                                self.current_dir = parent.to_path_buf();
                            }
                        } else {
                            self.current_dir = path;
                        }
                        self.refresh();
                        self.selected_idx = 0;
                        None
                    } else {
                        Some(path)
                    }
                } else {
                    None
                }
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh();
                    self.selected_idx = 0;
                }
                None
            }
            KeyCode::Tab => {
                self.input_focused = true;
                None
            }
            KeyCode::Char('h') if key.modifiers == KeyModifiers::ALT => {
                self.show_hidden = !self.show_hidden;
                self.refresh();
                None
            }
            KeyCode::Char('e') if key.modifiers == KeyModifiers::ALT => {
                self.detect_encoding = !self.detect_encoding;
                None
            }
            KeyCode::Char(c) if c.is_alphanumeric() => {
                // Typeahead
                let search = c.to_lowercase().to_string();
                if let Some(idx) = self.entries.iter().position(|e| e.name.to_lowercase().starts_with(&search)) {
                    self.selected_idx = idx;
                    if !self.entries[idx].is_dir {
                        self.input_text = self.entries[idx].name.clone();
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> Option<PathBuf> {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return None;
        }

        let (mx, my) = (mouse.column, mouse.row);

        // Options bar (Show Hidden / Detect Encoding)
        if my == y + 2 {
            let hidden_x = x + 2;
            let hidden_w = 20;
            if mx >= hidden_x && mx < hidden_x + hidden_w {
                self.show_hidden = !self.show_hidden;
                self.refresh();
                return None;
            }
            let enc_x = x + 25;
            let enc_w = 25;
            if mx >= enc_x && mx < enc_x + enc_w {
                self.detect_encoding = !self.detect_encoding;
                return None;
            }
        }

        // Quick-nav bar
        if my == y + 3 {
            let navs = ["..", "/", "~", "Docs", "Downloads"];
            let mut nx = x + 2;
            for nav in navs {
                let nav_len = nav.len() as u16 + 2;
                if mx >= nx && mx < nx + nav_len {
                    match nav {
                        ".." => {
                            if let Some(parent) = self.current_dir.parent() {
                                self.current_dir = parent.to_path_buf();
                            }
                        }
                        "/" => self.current_dir = PathBuf::from("/"),
                        "~" => {
                            if let Some(home) = std::env::var_os("HOME") {
                                self.current_dir = PathBuf::from(home);
                            }
                        }
                        "Docs" => {
                            if let Some(home) = std::env::var_os("HOME") {
                                self.current_dir = PathBuf::from(home).join("Documents");
                            }
                        }
                        "Downloads" => {
                            if let Some(home) = std::env::var_os("HOME") {
                                self.current_dir = PathBuf::from(home).join("Downloads");
                            }
                        }
                        _ => {}
                    }
                    self.refresh();
                    self.selected_idx = 0;
                    return None;
                }
                nx += nav_len + 1;
            }
        }

        // Header click sorting
        if my == y + 4 {
            let name_x = x + 2;
            let size_x = x + w - 20;
            let mod_x = x + w - 10;
            
            if mx >= name_x && mx < size_x {
                if self.sort_by == SortBy::Name {
                    self.sort_order = if self.sort_order == SortOrder::Ascending { SortOrder::Descending } else { SortOrder::Ascending };
                } else {
                    self.sort_by = SortBy::Name;
                    self.sort_order = SortOrder::Ascending;
                }
                self.sort();
            } else if mx >= size_x && mx < mod_x {
                if self.sort_by == SortBy::Size {
                    self.sort_order = if self.sort_order == SortOrder::Ascending { SortOrder::Descending } else { SortOrder::Ascending };
                } else {
                    self.sort_by = SortBy::Size;
                    self.sort_order = SortOrder::Ascending;
                }
                self.sort();
            } else if mx >= mod_x && mx < x + w - 2 {
                if self.sort_by == SortBy::Modified {
                    self.sort_order = if self.sort_order == SortOrder::Ascending { SortOrder::Descending } else { SortOrder::Ascending };
                } else {
                    self.sort_by = SortBy::Modified;
                    self.sort_order = SortOrder::Ascending;
                }
                self.sort();
            }
            return None;
        }

        // Entries
        let visible_count = h.saturating_sub(9) as usize;
        if my >= y + 5 && my < y + 5 + visible_count as u16 {
            let click_idx = (my - (y + 5)) as usize;
            let start_idx = if self.selected_idx >= visible_count {
                self.selected_idx - visible_count + 1
            } else {
                0
            };
            let idx = start_idx + click_idx;
            if idx < self.entries.len() {
                self.selected_idx = idx;
                self.input_focused = false;
                if let Some(entry) = self.entries.get(idx) {
                    if !entry.is_dir {
                        self.input_text = entry.name.clone();
                    }
                }
            }
        }

        // Input field
        if my == y + h - 2 {
            self.input_focused = true;
        }

        None
    }

    pub fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        // Render current dir
        let dir_str = self.current_dir.to_string_lossy();
        for (i, c) in dir_str.chars().enumerate() {
            if (i as u16) < w - 4 {
                renderer.set_cell(x + 2 + i as u16, y + 1, Cell { ch: c, ..Default::default() });
            }
        }

        // Options bar
        let hidden_text = format!("[{}] Show Hidden (Alt+H)", if self.show_hidden { "x" } else { " " });
        for (i, c) in hidden_text.chars().enumerate() {
            renderer.set_cell(x + 2 + i as u16, y + 2, Cell { ch: c, ..Default::default() });
        }
        let enc_text = format!("[{}] Detect Encoding (Alt+E)", if self.detect_encoding { "x" } else { " " });
        for (i, c) in enc_text.chars().enumerate() {
            renderer.set_cell(x + 25 + i as u16, y + 2, Cell { ch: c, ..Default::default() });
        }

        // Quick-nav bar
        let navs = ["..", "/", "~", "Docs", "Downloads"];
        let mut nx = x + 2;
        let ny = y + 3;
        for nav in navs {
            let nav_str = format!(" {} ", nav);
            for c in nav_str.chars() {
                renderer.set_cell(nx, ny, Cell { ch: c, bg: Color::DarkGrey, ..Default::default() });
                nx += 1;
            }
            nx += 1;
        }

        // Header
        let name_indicator = if self.sort_by == SortBy::Name { if self.sort_order == SortOrder::Ascending { " ▲" } else { " ▼" } } else { "" };
        let size_indicator = if self.sort_by == SortBy::Size { if self.sort_order == SortOrder::Ascending { " ▲" } else { " ▼" } } else { "" };
        let mod_indicator = if self.sort_by == SortBy::Modified { if self.sort_order == SortOrder::Ascending { " ▲" } else { " ▼" } } else { "" };

        let name_head = format!("Name{}", name_indicator);
        let size_head = format!("Size{}", size_indicator);
        let mod_head = format!("Modified{}", mod_indicator);

        for (i, c) in name_head.chars().enumerate() {
            renderer.set_cell(x + 2 + i as u16, y + 4, Cell { ch: c, ..Default::default() });
        }
        for (i, c) in size_head.chars().enumerate() {
            renderer.set_cell(x + w - 20 + i as u16, y + 4, Cell { ch: c, ..Default::default() });
        }
        for (i, c) in mod_head.chars().enumerate() {
            renderer.set_cell(x + w - 10 + i as u16, y + 4, Cell { ch: c, ..Default::default() });
        }

        // Entries
        let visible_count = h.saturating_sub(9) as usize;
        let start_idx = if self.selected_idx >= visible_count {
            self.selected_idx - visible_count + 1
        } else {
            0
        };

        for i in 0..visible_count {
            let idx = start_idx + i;
            if let Some(entry) = self.entries.get(idx) {
                let iy = y + 5 + i as u16;
                let is_selected = idx == self.selected_idx && !self.input_focused;
                let bg = if is_selected { Color::White } else { Color::Reset };
                let fg = if is_selected { Color::Black } else { Color::White };

                for dx in 1..w - 1 {
                    renderer.set_cell(x + dx, iy, Cell { ch: ' ', bg, ..Default::default() });
                }

                let name = if entry.is_dir && entry.name != ".." {
                    format!("{}/", entry.name)
                } else {
                    entry.name.clone()
                };

                for (j, c) in name.chars().enumerate() {
                    if (j as u16) < w - 22 {
                        renderer.set_cell(x + 2 + j as u16, iy, Cell { ch: c, bg, fg, ..Default::default() });
                    }
                }

                if !entry.is_dir {
                    let size_str = if entry.size < 1024 {
                        format!("{} B", entry.size)
                    } else if entry.size < 1024 * 1024 {
                        format!("{:.1} KB", entry.size as f64 / 1024.0)
                    } else {
                        format!("{:.1} MB", entry.size as f64 / (1024.0 * 1024.0))
                    };
                    for (j, c) in size_str.chars().enumerate() {
                        if (j as u16) < 8 {
                            renderer.set_cell(x + w - 20 + j as u16, iy, Cell { ch: c, bg, fg, ..Default::default() });
                        }
                    }

                    if let Some(m) = entry.modified {
                        let now = SystemTime::now();
                        let duration = now.duration_since(m).unwrap_or(std::time::Duration::from_secs(0));
                        let mod_str = if duration.as_secs() < 24 * 3600 {
                            if duration.as_secs() < 60 {
                                "just now".to_string()
                            } else if duration.as_secs() < 3600 {
                                format!("{}m ago", duration.as_secs() / 60)
                            } else {
                                format!("{}h ago", duration.as_secs() / 3600)
                            }
                        } else {
                            let datetime: DateTime<Local> = m.into();
                            datetime.format("%Y-%m-%d").to_string()
                        };
                        for (j, c) in mod_str.chars().enumerate() {
                            if (j as u16) < 10 {
                                renderer.set_cell(x + w - 10 + j as u16, iy, Cell { ch: c, bg, fg, ..Default::default() });
                            }
                        }
                    }
                }
            }
        }

        // Input field
        let iy = y + h - 2;
        let bg = if self.input_focused { Color::White } else { Color::Reset };
        let fg = if self.input_focused { Color::Black } else { Color::White };
        
        let label = "File name: ";
        for (i, c) in label.chars().enumerate() {
            renderer.set_cell(x + 2 + i as u16, iy, Cell { ch: c, ..Default::default() });
        }
        
        let input_x = x + 2 + label.len() as u16;
        let input_w = w - label.len() as u16 - 4;
        for dx in 0..input_w {
            renderer.set_cell(input_x + dx, iy, Cell { ch: ' ', bg, ..Default::default() });
        }
        for (i, c) in self.input_text.chars().enumerate() {
            if (i as u16) < input_w {
                renderer.set_cell(input_x + i as u16, iy, Cell { ch: c, bg, fg, ..Default::default() });
            }
        }
    }
}

pub fn render_base_dialog(renderer: &mut Renderer, title: &str, x: u16, y: u16, w: u16, h: u16) {
    let bg = Color::Reset;
    let _fg = Color::White;
    let border_fg = Color::White;

    // Fill background
    for dy in 0..h {
        for dx in 0..w {
            renderer.set_cell(x + dx, y + dy, Cell {
                ch: ' ',
                bg,
                ..Default::default()
            });
        }
    }

    // Draw borders (Unicode box-drawing)
    renderer.set_cell(x, y, Cell { ch: '┌', fg: border_fg, bg, ..Default::default() });
    renderer.set_cell(x + w - 1, y, Cell { ch: '┐', fg: border_fg, bg, ..Default::default() });
    renderer.set_cell(x, y + h - 1, Cell { ch: '└', fg: border_fg, bg, ..Default::default() });
    renderer.set_cell(x + w - 1, y + h - 1, Cell { ch: '┘', fg: border_fg, bg, ..Default::default() });

    for dx in 1..w - 1 {
        renderer.set_cell(x + dx, y, Cell { ch: '─', fg: border_fg, bg, ..Default::default() });
        renderer.set_cell(x + dx, y + h - 1, Cell { ch: '─', fg: border_fg, bg, ..Default::default() });
    }

    for dy in 1..h - 1 {
        renderer.set_cell(x, y + dy, Cell { ch: '│', fg: border_fg, bg, ..Default::default() });
        renderer.set_cell(x + w - 1, y + dy, Cell { ch: '│', fg: border_fg, bg, ..Default::default() });
    }

    let title_str = format!(" {} ", title);
    for (i, c) in title_str.chars().enumerate() {
        if 2 + (i as u16) < w - 2 {
            renderer.set_cell(x + 2 + (i as u16), y, Cell { ch: c, fg: border_fg, bg, ..Default::default() });
        }
    }
}

pub struct OpenDialog {
    pub browser: FileBrowser,
    pub error_message: Option<String>,
}

impl OpenDialog {
    pub fn new() -> Self {
        Self {
            browser: FileBrowser::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))),
            error_message: None,
        }
    }
}

impl Dialog for OpenDialog {
    fn title(&self) -> &str {
        "Open File"
    }

    fn dimensions(&self) -> (u16, u16) {
        (60, 22)
    }

    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        render_base_dialog(renderer, self.title(), x, y, w, h);
        self.browser.render(renderer, x, y, w, h);

        if let Some(ref msg) = self.error_message {
            let msg_x = x + 2;
            let msg_y = y + h - 3;
            for (i, c) in msg.chars().enumerate() {
                if (i as u16) < w - 4 {
                    renderer.set_cell(msg_x + i as u16, msg_y, Cell { ch: c, fg: Color::Red, ..Default::default() });
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action> {
        if let Some(path) = self.browser.handle_key(key) {
            return DialogResult::Ok(Action::ConfirmPath(path));
        }
        match key.code {
            KeyCode::Esc => DialogResult::Cancel,
            _ => DialogResult::Pending,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action> {
        if let Some(path) = self.browser.handle_mouse(mouse, x, y, w, h) {
            return DialogResult::Ok(Action::ConfirmPath(path));
        }
        DialogResult::Pending
    }

    fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }
}

pub struct SaveAsDialog {
    pub browser: FileBrowser,
    pub error_message: Option<String>,
}

impl SaveAsDialog {
    pub fn new(current_path: Option<&PathBuf>) -> Self {
        let mut browser = FileBrowser::new(
            current_path.and_then(|p| p.parent()).map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))));
        
        if let Some(path) = current_path {
            if let Some(name) = path.file_name() {
                browser.input_text = name.to_string_lossy().to_string();
            }
        }
        
        browser.input_focused = true;
        
        Self { 
            browser,
            error_message: None,
        }
    }
}

impl Dialog for SaveAsDialog {
    fn title(&self) -> &str {
        "Save As..."
    }

    fn dimensions(&self) -> (u16, u16) {
        (60, 22)
    }

    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        render_base_dialog(renderer, self.title(), x, y, w, h);
        self.browser.render(renderer, x, y, w, h);

        if let Some(ref msg) = self.error_message {
            let msg_x = x + 2;
            let msg_y = y + h - 3;
            for (i, c) in msg.chars().enumerate() {
                if (i as u16) < w - 4 {
                    renderer.set_cell(msg_x + i as u16, msg_y, Cell { ch: c, fg: Color::Red, ..Default::default() });
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action> {
        if let Some(path) = self.browser.handle_key(key) {
            return DialogResult::Ok(Action::ConfirmPath(path));
        }
        match key.code {
            KeyCode::Esc => DialogResult::Cancel,
            _ => DialogResult::Pending,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action> {
        if let Some(path) = self.browser.handle_mouse(mouse, x, y, w, h) {
            return DialogResult::Ok(Action::ConfirmPath(path));
        }
        DialogResult::Pending
    }

    fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }
}

pub struct MessageDialog {
    pub title: String,
    pub message: String,
    pub buttons: Vec<(String, Action)>,
    pub selected_btn: usize,
}

impl MessageDialog {
    pub fn new(title: String, message: String, buttons: Vec<(String, Action)>) -> Self {
        Self {
            title,
            message,
            buttons,
            selected_btn: 0,
        }
    }
}

impl Dialog for MessageDialog {
    fn title(&self) -> &str {
        &self.title
    }

    fn dimensions(&self) -> (u16, u16) {
        (40, 8)
    }

    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        render_base_dialog(renderer, self.title(), x, y, w, h);

        let lx = x + (w.saturating_sub(self.message.len() as u16)) / 2;
        let ly = y + 2;
        for (i, c) in self.message.chars().enumerate() {
            renderer.set_cell(lx + i as u16, ly, Cell { ch: c, ..Default::default() });
        }

        // Buttons
        let total_btns_width: u16 = self.buttons.iter().map(|(s, _)| s.len() as u16 + 4).sum::<u16>() + (self.buttons.len() as u16 - 1);
        let mut bx = x + (w.saturating_sub(total_btns_width)) / 2;
        let by = y + h - 2;

        for (i, (s, _)) in self.buttons.iter().enumerate() {
            let btn_text = format!("[ {} ]", s);
            let is_selected = i == self.selected_btn;
            let bg = if is_selected { Color::White } else { Color::Reset };
            let fg = if is_selected { Color::Black } else { Color::White };

            for c in btn_text.chars() {
                renderer.set_cell(bx, by, Cell { ch: c, bg, fg, ..Default::default() });
                bx += 1;
            }
            bx += 1; // Spacer
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action> {
        match key.code {
            KeyCode::Left => {
                if self.selected_btn > 0 {
                    self.selected_btn -= 1;
                }
                DialogResult::Pending
            }
            KeyCode::Right => {
                if self.selected_btn + 1 < self.buttons.len() {
                    self.selected_btn += 1;
                }
                DialogResult::Pending
            }
            KeyCode::Enter => DialogResult::Ok(self.buttons[self.selected_btn].1.clone()),
            KeyCode::Esc => DialogResult::Cancel,
            _ => DialogResult::Pending,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action> {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return DialogResult::Pending;
        }

        let (mx, my) = (mouse.column, mouse.row);
        if my == y + h - 2 {
            let total_btns_width: u16 = self.buttons.iter().map(|(s, _)| s.len() as u16 + 4).sum::<u16>() + (self.buttons.len() as u16 - 1);
            let mut bx = x + (w.saturating_sub(total_btns_width)) / 2;
            for (i, (s, _)) in self.buttons.iter().enumerate() {
                let btn_len = s.len() as u16 + 4;
                if mx >= bx && mx < bx + btn_len {
                    return DialogResult::Ok(self.buttons[i].1.clone());
                }
                bx += btn_len + 1;
            }
        }
        DialogResult::Pending
    }
}

pub struct AboutDialog {
    pub i18n_about: String,
    pub i18n_ok: String,
}

impl AboutDialog {
    pub fn new(i18n: &led_core::I18n) -> Self {
        Self {
            i18n_about: i18n.get("menu.help.about").to_string(),
            i18n_ok: "OK".to_string(),
        }
    }
}

impl Dialog for AboutDialog {
    fn title(&self) -> &str {
        &self.i18n_about
    }

    fn dimensions(&self) -> (u16, u16) {
        (40, 10)
    }

    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        render_base_dialog(renderer, self.title(), x, y, w, h);

        let content = [
            "led editor v0.1.0",
            "A lightweight, modern TUI editor.",
            "",
            "License: MIT",
        ];

        for (i, line) in content.iter().enumerate() {
            let lx = x + (w.saturating_sub(line.len() as u16)) / 2;
            let ly = y + 2 + i as u16;
            for (j, c) in line.chars().enumerate() {
                renderer.set_cell(lx + j as u16, ly, Cell { ch: c, ..Default::default() });
            }
        }

        let btn_text = format!("[ {} ]", self.i18n_ok);
        let bx = x + (w.saturating_sub(btn_text.len() as u16)) / 2;
        let by = y + h - 2;
        for (i, c) in btn_text.chars().enumerate() {
            renderer.set_cell(bx + i as u16, by, Cell {
                ch: c,
                bg: Color::White,
                fg: Color::Black,
                ..Default::default()
            });
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => DialogResult::Ok(Action::Confirm),
            _ => DialogResult::Pending,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action> {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return DialogResult::Pending;
        }
        let (mx, my) = (mouse.column, mouse.row);
        let btn_text = format!("[ {} ]", self.i18n_ok);
        let bx = x + (w.saturating_sub(btn_text.len() as u16)) / 2;
        let by = y + h - 2;
        if my == by && mx >= bx && mx < bx + btn_text.len() as u16 {
            return DialogResult::Ok(Action::Confirm);
        }
        DialogResult::Pending
    }
}

pub struct ReopenConfirmationDialog {
    pub i18n_title: String,
    pub i18n_message: String,
    pub i18n_discard: String,
    pub i18n_cancel: String,
    pub selected_btn: usize,
}

impl ReopenConfirmationDialog {
    pub fn new(i18n: &led_core::I18n) -> Self {
        Self {
            i18n_title: "Reopen File".to_string(),
            i18n_message: "Discard unsaved changes and reopen?".to_string(),
            i18n_discard: i18n.get("dialog.discard").to_string(),
            i18n_cancel: i18n.get("dialog.cancel").to_string(),
            selected_btn: 0,
        }
    }
}

impl Dialog for ReopenConfirmationDialog {
    fn title(&self) -> &str {
        &self.i18n_title
    }

    fn dimensions(&self) -> (u16, u16) {
        (40, 8)
    }

    fn render(&self, renderer: &mut Renderer, x: u16, y: u16, w: u16, h: u16) {
        render_base_dialog(renderer, self.title(), x, y, w, h);

        let lx = x + (w.saturating_sub(self.i18n_message.len() as u16)) / 2;
        let ly = y + 2;
        for (i, c) in self.i18n_message.chars().enumerate() {
            renderer.set_cell(lx + i as u16, ly, Cell { ch: c, ..Default::default() });
        }

        let buttons = [&self.i18n_discard, &self.i18n_cancel];
        let total_btns_width: u16 = buttons.iter().map(|s| s.len() as u16 + 4).sum::<u16>() + 1;
        let mut bx = x + (w.saturating_sub(total_btns_width)) / 2;
        let by = y + h - 2;

        for (i, s) in buttons.iter().enumerate() {
            let btn_text = format!("[ {} ]", s);
            let is_selected = i == self.selected_btn;
            let bg = if is_selected { Color::White } else { Color::Reset };
            let fg = if is_selected { Color::Black } else { Color::White };

            for c in btn_text.chars() {
                renderer.set_cell(bx, by, Cell { ch: c, bg, fg, ..Default::default() });
                bx += 1;
            }
            bx += 1;
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> DialogResult<Action> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.selected_btn = 1 - self.selected_btn;
                DialogResult::Pending
            }
            KeyCode::Enter => {
                if self.selected_btn == 0 {
                    DialogResult::Ok(Action::Discard)
                } else {
                    DialogResult::Cancel
                }
            }
            KeyCode::Esc => DialogResult::Cancel,
            _ => DialogResult::Pending,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, x: u16, y: u16, w: u16, h: u16) -> DialogResult<Action> {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return DialogResult::Pending;
        }
        let (mx, my) = (mouse.column, mouse.row);
        if my == y + h - 2 {
            let buttons = [&self.i18n_discard, &self.i18n_cancel];
            let total_btns_width: u16 = buttons.iter().map(|s| s.len() as u16 + 4).sum::<u16>() + 1;
            let mut bx = x + (w.saturating_sub(total_btns_width)) / 2;
            for (i, s) in buttons.iter().enumerate() {
                let btn_len = s.len() as u16 + 4;
                if mx >= bx && mx < bx + btn_len {
                    if i == 0 { return DialogResult::Ok(Action::Discard); }
                    else { return DialogResult::Cancel; }
                }
                bx += btn_len + 1;
            }
        }
        DialogResult::Pending
    }
}

