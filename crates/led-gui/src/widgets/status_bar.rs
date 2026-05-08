use gpui::*;
use crate::workspace::Workspace;
use led_core::theme::{Color as LedColor};

pub struct StatusBar {
    workspace: Entity<Workspace>,
}

impl StatusBar {
    pub fn new(workspace: Entity<Workspace>, _cx: &mut Context<Self>) -> Self {
        Self { workspace }
    }

    fn led_color_to_gpui(&self, color: LedColor) -> Rgba {
        rgb((color.0 as u32) << 16 | (color.1 as u32) << 8 | (color.2 as u32))
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let theme = &workspace.theme;
        let editor = workspace.active_editor();
        
        let file_name = editor.path.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("[No Name]");
        
        let modified_flag = if editor.is_modified() { " [+]" } else { "" };
        let vi_mode = format!("{:?}", editor.vi_mode).to_uppercase();
        
        let (line, col) = editor.char_to_line_col(editor.cursor);

        let selection_info = if let Some(ref sel) = editor.selection {
            format!("{} chars", sel.end - sel.start)
        } else {
            "".to_string()
        };

        let encoding = format!("{:?}", editor.encoding).to_uppercase();
        let line_ending = format!("{:?}", editor.line_ending).to_uppercase();
        let syntax = editor.syntax_highlighter.as_ref().map(|h| h.def.meta.name.clone()).unwrap_or("Plain Text".to_string());

        div()
            .h_6()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .bg(self.led_color_to_gpui(theme.ui.status_bar_bg))
            .text_color(self.led_color_to_gpui(theme.ui.status_bar_fg))
            .border_t_1()
            .border_color(self.led_color_to_gpui(theme.editor.line_number))
            .text_sm()
            .child(
                div()
                    .px_2()
                    .child(format!("{}{}", file_name, modified_flag))
            )
            .child(
                div()
                    .px_2()
                    .flex()
                    .gap_4()
                    .child(selection_info)
                    .child(vi_mode)
                    .child(format!("Ln {}, Col {}", line + 1, col + 1))
                    .child(encoding)
                    .child(line_ending)
                    .child(syntax)
            )
    }
}
