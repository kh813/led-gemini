use gpui::*;
use crate::workspace::Workspace;
use led_core::syntax::TokenType;
use led_core::theme::{Color as LedColor, Theme};

pub struct EditorView {
    workspace: Entity<Workspace>,
    focus_handle: FocusHandle,
    last_click_at: Option<std::time::Instant>,
    click_count: usize,
}

impl EditorView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            last_click_at: None,
            click_count: 0,
        }
    }

    fn led_color_to_gpui(&self, color: LedColor) -> Rgba {
        rgb((color.0 as u32) << 16 | (color.1 as u32) << 8 | (color.2 as u32))
    }

    fn token_color(&self, token_type: TokenType, theme: &Theme) -> Rgba {
        let color = match token_type {
            TokenType::Keyword => theme.syntax.keyword,
            TokenType::TypeName => theme.syntax.type_name,
            TokenType::Function => theme.syntax.function,
            TokenType::String => theme.syntax.string,
            TokenType::Number => theme.syntax.number,
            TokenType::Comment => theme.syntax.comment,
            TokenType::Operator => theme.syntax.operator,
            TokenType::Punctuation => theme.syntax.punctuation,
            TokenType::Constant => theme.syntax.constant,
            TokenType::Attribute => theme.syntax.attribute,
            TokenType::Error => theme.syntax.error,
        };
        self.led_color_to_gpui(color.unwrap_or(theme.editor.foreground))
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke.key;
        let shift = event.keystroke.modifiers.shift;

        self.workspace.update(cx, |w, _| {
            let editor = w.active_editor_mut();
            match key.as_str() {
                "up" => editor.move_cursor_up(shift),
                "down" => editor.move_cursor_down(shift),
                "left" => editor.move_cursor_left(shift),
                "right" => editor.move_cursor_right(shift),
                "home" => editor.move_cursor_home(shift),
                "end" => editor.move_cursor_end(shift),
                "backspace" => {
                    if let Some(range) = editor.selection.clone() {
                        editor.delete(range);
                    } else if editor.cursor > 0 {
                        editor.delete(editor.cursor - 1..editor.cursor);
                    }
                }
                "enter" => {
                    if let Some(range) = editor.selection.clone() {
                        editor.delete(range);
                    }
                    editor.insert(editor.cursor, "\n");
                }
                k if k.len() == 1 => {
                    if let Some(range) = editor.selection.clone() {
                        editor.delete(range);
                    }
                    editor.insert(editor.cursor, k);
                }
                _ => {}
            }
        });
        cx.notify();
    }

    fn mouse_pos_to_char_pos(&self, position: Point<Pixels>, cx: &mut Context<Self>) -> usize {
        let workspace = self.workspace.read(cx);
        let editor = workspace.active_editor();
        
        let line_height = px(20.0);
        let gutter_width = px(50.0);
        let char_width = px(8.4); // Rough estimate for Menlo 14pt

        let relative_y = position.y - px(32.0) - px(32.0); // Offset by menu and tab bar
        let line_idx = (relative_y / line_height).floor() as i32 + editor.scroll_row as i32;
        let line_idx = line_idx.max(0).min(editor.line_count() as i32 - 1) as usize;

        let relative_x = position.x - gutter_width + px(editor.scroll_col as f32 * 8.4);
        let col_idx = (relative_x / char_width).round() as i32;
        let col_idx = col_idx.max(0) as usize;

        editor.line_col_to_char(line_idx, col_idx)
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        let now = std::time::Instant::now();
        if let Some(last) = self.last_click_at {
            if now.duration_since(last).as_millis() < 300 {
                self.click_count += 1;
            } else {
                self.click_count = 1;
            }
        } else {
            self.click_count = 1;
        }
        self.last_click_at = Some(now);

        let char_pos = self.mouse_pos_to_char_pos(event.position, cx);
        let gutter_width = px(50.0);
        let is_gutter_click = event.position.x < gutter_width;

        self.workspace.update(cx, |w, _| {
            let editor = w.active_editor_mut();
            if is_gutter_click {
                let (line, _) = editor.char_to_line_col(char_pos);
                editor.select_line(line);
            } else {
                match self.click_count {
                    1 => {
                        editor.cursor = char_pos;
                        if event.modifiers.shift {
                            editor.ensure_selection();
                            editor.update_selection();
                        } else {
                            editor.selection = None;
                            editor.selection_anchor = Some(char_pos);
                        }
                    }
                    2 => editor.select_word(char_pos),
                    3 => {
                        let (line, _) = editor.char_to_line_col(char_pos);
                        editor.select_line(line);
                    }
                    _ => {
                        editor.cursor = char_pos;
                        editor.selection = None;
                        editor.selection_anchor = Some(char_pos);
                    }
                }
            }
        });
        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.pressed_button.is_some() {
            let char_pos = self.mouse_pos_to_char_pos(event.position, cx);
            self.workspace.update(cx, |w, _| {
                let editor = w.active_editor_mut();
                editor.cursor = char_pos;
                editor.ensure_selection();
                editor.update_selection();
            });
            cx.notify();
        }
    }

    fn handle_scroll(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            let editor = w.active_editor_mut();
            let delta = event.delta.pixel_delta(px(20.0));
            
            // Vertical scroll
            if delta.y != px(0.0) {
                let rows = (delta.y / px(20.0)).floor() as i32;
                if rows > 0 {
                    editor.scroll_row = editor.scroll_row.saturating_sub(rows as usize);
                } else {
                    editor.scroll_row = (editor.scroll_row + (-rows) as usize).min(editor.line_count().saturating_sub(1));
                }
            }

            // Horizontal scroll
            if delta.x != px(0.0) {
                let cols = (delta.x / px(8.4)).floor() as i32;
                if cols > 0 {
                    editor.scroll_col = editor.scroll_col.saturating_sub(cols as usize);
                } else {
                    editor.scroll_col += (-cols) as usize;
                }
            }
        });
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let theme = &workspace.theme;

        div()
            .track_focus(&self.focus_handle)
            .key_context("Editor")
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_key_down(event, window, cx);
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx| {
                this.handle_mouse_down(event, window, cx);
            }))
            .on_mouse_move(cx.listener(|this, event, window, cx| {
                this.handle_mouse_move(event, window, cx);
            }))
            .on_scroll_wheel(cx.listener(|this, event, window, cx| {
                this.handle_scroll(event, window, cx);
            }))
            .w_full()
            .h_full()
            .bg(self.led_color_to_gpui(theme.editor.background))
            .text_color(self.led_color_to_gpui(theme.editor.foreground))
            .font_family("Menlo")
            .text_size(px(14.0))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(self.render_lines(workspace))
            )
    }
}

impl EditorView {
    fn render_lines(&self, workspace: &Workspace) -> impl IntoElement {
        let editor = workspace.active_editor();
        let line_count = editor.line_count();
        let scroll_row = editor.scroll_row;
        
        let word_wrap = false; // Placeholder

        div()
            .flex_col()
            .children(
                (scroll_row..line_count.min(scroll_row + 100)).map(|idx| {
                    if word_wrap {
                        self.render_wrapped_line(idx, workspace).into_any_element()
                    } else {
                        self.render_line(idx, workspace).into_any_element()
                    }
                })
            )
    }

    fn render_wrapped_line(&self, line_idx: usize, workspace: &Workspace) -> impl IntoElement {
        let editor = workspace.active_editor();
        let theme = &workspace.theme;
        let line = editor.rope.line(line_idx);
        let line_str = line.to_string();

        // Assume char_width is 8.4, we need to know the window width
        // For simplicity, let's hardcode a width or use a reasonable default
        let wraps = editor.wrap_line(line_idx, 80, 4); 

        div()
            .flex_col()
            .children(wraps.into_iter().enumerate().map(|(vidx, range)| {
                let chunk = &line_str[range.start..range.end];
                div()
                    .flex()
                    .h(px(20.0))
                    .child(
                        div()
                            .w(px(50.0))
                            .justify_end()
                            .px_2()
                            .text_color(self.led_color_to_gpui(theme.editor.line_number))
                            .child(if vidx == 0 { (line_idx + 1).to_string() } else { "".to_string() })
                    )
                    .child(
                        div()
                            .child(chunk.to_string())
                    )
            }))
    }

    fn render_line(&self, line_idx: usize, workspace: &Workspace) -> impl IntoElement {
        let editor = workspace.active_editor();
        let theme = &workspace.theme;

        let line = editor.rope.line(line_idx);
        let line_str = line.to_string();
        
        let (cursor_line, cursor_col) = editor.char_to_line_col(editor.cursor);
        let is_cursor_line = line_idx == cursor_line;

        let bg_hsla: Hsla = hsla(0., 0., 0., 0.);
        let bg: Rgba = if is_cursor_line {
            theme.editor.current_line.map(|c| self.led_color_to_gpui(c)).unwrap_or(bg_hsla.into())
        } else {
            bg_hsla.into()
        };

        let gutter_width = px(50.0);

        div()
            .flex()
            .h(px(20.0))
            .bg(bg)
            .child(
                div()
                    .w(gutter_width)
                    .justify_end()
                    .px_2()
                    .text_color(self.led_color_to_gpui(theme.editor.line_number))
                    .child((line_idx + 1).to_string())
            )
            .child(
                div()
                    .flex()
                    .h_full()
                    .relative()
                    .ml(px(-(editor.scroll_col as f32 * 8.4)))
                    .children(self.render_line_content(line_idx, &line_str, workspace))
                    .child(if is_cursor_line {
                        // Render cursor
                        div()
                            .absolute()
                            .top_0()
                            .left(px(cursor_col as f32 * 8.4))
                            .w(px(2.0))
                            .h_full()
                            .bg(self.led_color_to_gpui(theme.editor.cursor))
                    } else {
                        div()
                    })
            )
    }

    fn render_line_content(&self, line_idx: usize, line_str: &str, workspace: &Workspace) -> Vec<impl IntoElement> {
        let editor = workspace.active_editor();
        let theme = &workspace.theme;

        let selection = editor.selection.clone();
        let line_start_char = editor.rope.line_to_char(line_idx);

        let mut elements = Vec::new();

        // Helper to render a chunk of text with potential selection highlight
        let mut render_chunk = |text: &str, start_char: usize, token_color: Option<Rgba>| {
            let chunk_chars: Vec<char> = text.chars().collect();
            let chunk_len = chunk_chars.len();
            
            if let Some(ref sel) = selection {
                let sel_start = if sel.start > start_char { sel.start - start_char } else { 0 };
                let sel_end = if sel.end > start_char { sel.end - start_char } else { 0 };

                if sel_start < chunk_len && sel_end > 0 {
                    let highlight_start = sel_start;
                    let highlight_end = sel_end.min(chunk_len);

                    if highlight_start > 0 {
                        elements.push(
                            div()
                                .text_color(token_color.unwrap_or(self.led_color_to_gpui(theme.editor.foreground)))
                                .child(chunk_chars[..highlight_start].iter().collect::<String>())
                        );
                    }

                    elements.push(
                        div()
                            .bg(self.led_color_to_gpui(theme.editor.selection))
                            .text_color(token_color.unwrap_or(self.led_color_to_gpui(theme.editor.foreground)))
                            .child(chunk_chars[highlight_start..highlight_end].iter().collect::<String>())
                    );

                    if highlight_end < chunk_len {
                        elements.push(
                            div()
                                .text_color(token_color.unwrap_or(self.led_color_to_gpui(theme.editor.foreground)))
                                .child(chunk_chars[highlight_end..].iter().collect::<String>())
                        );
                    }
                    return;
                }
            }

            elements.push(
                div()
                    .text_color(token_color.unwrap_or(self.led_color_to_gpui(theme.editor.foreground)))
                    .child(text.to_string())
            );
        };

        if let Some(Some(tokens)) = editor.line_tokens.get(line_idx) {
            let mut last_offset = 0;
            for token in tokens {
                if token.byte_range.start > last_offset {
                    let text = &line_str[last_offset..token.byte_range.start];
                    let start_char = line_start_char + line_str[..last_offset].chars().count();
                    render_chunk(text, start_char, None);
                }
                let text = &line_str[token.byte_range.clone()];
                let start_char = line_start_char + line_str[..token.byte_range.start].chars().count();
                render_chunk(text, start_char, Some(self.token_color(token.token, theme)));
                last_offset = token.byte_range.end;
            }
            if last_offset < line_str.len() {
                let text = &line_str[last_offset..];
                let start_char = line_start_char + line_str[..last_offset].chars().count();
                render_chunk(text, start_char, None);
            }
        } else {
            render_chunk(line_str, line_start_char, None);
        }
        elements
    }
}
