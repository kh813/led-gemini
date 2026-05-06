use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::config::Config;

pub struct I18n {
    strings: HashMap<String, String>,
}

impl I18n {
    pub fn load(lang: &str) -> Self {
        let strings = Self::get_en_defaults();
        
        if lang != "en" {
            if let Some(path) = Self::locale_file_path(lang) {
                if path.exists() {
                    if let Ok(_content) = fs::read_to_string(path) {
                        // TODO: Parse TOML and merge
                    }
                }
            }
        }
        
        Self { strings }
    }

    fn get_en_defaults() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("menu.file".to_string(), "File".to_string());
        m.insert("menu.file.new".to_string(), "New".to_string());
        m.insert("menu.file.open".to_string(), "Open…".to_string());
        m.insert("menu.file.save".to_string(), "Save".to_string());
        m.insert("menu.file.save_as".to_string(), "Save As…".to_string());
        m.insert("menu.file.close".to_string(), "Close".to_string());
        m.insert("menu.file.exit".to_string(), "Exit".to_string());

        m.insert("menu.edit".to_string(), "Edit".to_string());
        m.insert("menu.edit.undo".to_string(), "Undo".to_string());
        m.insert("menu.edit.redo".to_string(), "Redo".to_string());
        m.insert("menu.edit.cut".to_string(), "Cut".to_string());
        m.insert("menu.edit.copy".to_string(), "Copy".to_string());
        m.insert("menu.edit.paste".to_string(), "Paste".to_string());
        m.insert("menu.edit.find".to_string(), "Find…".to_string());
        m.insert("menu.edit.replace".to_string(), "Replace…".to_string());
        m.insert("menu.edit.select_all".to_string(), "Select All".to_string());

        m.insert("menu.view".to_string(), "View".to_string());
        m.insert("menu.view.go_to_line".to_string(), "Go to Line…".to_string());
        m.insert("menu.view.line_numbers".to_string(), "Line Numbers".to_string());
        m.insert("menu.view.word_wrap".to_string(), "Word Wrap".to_string());
        m.insert("menu.view.vi_mode".to_string(), "Vi Mode".to_string());
        
        m.insert("menu.help.about".to_string(), "About".to_string());

        m.insert("dialog.unsaved.title".to_string(), "Unsaved Changes".to_string());
        m.insert("dialog.unsaved.message".to_string(), "Save changes before closing?".to_string());
        m.insert("dialog.save".to_string(), "Save".to_string());
        m.insert("dialog.dont_save".to_string(), "Don't Save".to_string());
        m.insert("dialog.discard".to_string(), "Discard".to_string());
        m.insert("dialog.cancel".to_string(), "Cancel".to_string());
        m
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    fn locale_file_path(lang: &str) -> Option<PathBuf> {
        Config::config_dir().map(|dir| dir.join("locales").join(format!("{}.toml", lang)))
    }
}
