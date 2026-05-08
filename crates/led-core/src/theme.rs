use serde::{Deserialize, Serialize, Deserializer, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color(r, g, b))
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::from_hex(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid hex color: {}", s)))
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub editor: EditorColors,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
}

impl Theme {
    pub fn builtins() -> Vec<Self> {
        let files = [
            include_str!("../../../assets/themes/catppuccin-latte.toml"),
            include_str!("../../../assets/themes/catppuccin-mocha.toml"),
            include_str!("../../../assets/themes/light.toml"),
            include_str!("../../../assets/themes/solarized-dark.toml"),
            include_str!("../../../assets/themes/solarized-light.toml"),
            include_str!("../../../assets/themes/tokyo-night.toml"),
        ];

        files
            .iter()
            .map(|s| toml::from_str(s).expect("failed to parse builtin theme"))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorColors {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub line_number: Color,
    pub current_line: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub menu_bar_bg: Color,
    pub menu_bar_fg: Color,
    pub menu_item_active_bg: Color,
    pub menu_item_active_fg: Color,
    pub tab_bar_bg: Color,
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,
    pub tab_inactive_bg: Color,
    pub tab_inactive_fg: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub panel_bg: Color,
    pub panel_fg: Color,
    pub panel_error_fg: Color,
    pub dialog_bg: Color,
    pub dialog_border: Color,
    pub button_active_bg: Color,
    pub button_active_fg: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: Option<Color>,
    pub type_name: Option<Color>,
    pub function: Option<Color>,
    pub string: Option<Color>,
    pub number: Option<Color>,
    pub comment: Option<Color>,
    pub operator: Option<Color>,
    pub punctuation: Option<Color>,
    pub constant: Option<Color>,
    pub attribute: Option<Color>,
    pub error: Option<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        // Tokyo Night inspired default
        Self {
            meta: ThemeMeta {
                name: "Tokyo Night".to_string(),
                author: Some("led contributors".to_string()),
                version: Some("1.0".to_string()),
            },
            editor: EditorColors {
                background: Color(26, 27, 38),
                foreground: Color(192, 202, 245),
                cursor: Color(192, 202, 245),
                selection: Color(40, 52, 87),
                line_number: Color(59, 66, 97),
                current_line: Some(Color(30, 32, 48)),
            },
            ui: UiColors {
                menu_bar_bg: Color(22, 22, 30),
                menu_bar_fg: Color(192, 202, 245),
                menu_item_active_bg: Color(122, 162, 247),
                menu_item_active_fg: Color(26, 27, 38),
                tab_bar_bg: Color(22, 22, 30),
                tab_active_bg: Color(26, 27, 38),
                tab_active_fg: Color(192, 202, 245),
                tab_inactive_bg: Color(22, 22, 30),
                tab_inactive_fg: Color(86, 95, 137),
                status_bar_bg: Color(22, 22, 30),
                status_bar_fg: Color(192, 202, 245),
                panel_bg: Color(22, 22, 30),
                panel_fg: Color(192, 202, 245),
                panel_error_fg: Color(247, 118, 142),
                dialog_bg: Color(30, 32, 48),
                dialog_border: Color(122, 162, 247),
                button_active_bg: Color(122, 162, 247),
                button_active_fg: Color(26, 27, 38),
            },
            syntax: SyntaxColors {
                keyword: Some(Color(187, 154, 247)),
                type_name: Some(Color(42, 195, 222)),
                function: Some(Color(122, 162, 247)),
                string: Some(Color(158, 206, 106)),
                number: Some(Color(255, 158, 100)),
                comment: Some(Color(86, 95, 137)),
                operator: Some(Color(137, 221, 255)),
                punctuation: Some(Color(192, 202, 245)),
                constant: Some(Color(255, 158, 100)),
                attribute: Some(Color(187, 154, 247)),
                error: Some(Color(247, 118, 142)),
            },
        }
    }
}
