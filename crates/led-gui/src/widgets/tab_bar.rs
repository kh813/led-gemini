use gpui::*;
use crate::workspace::Workspace;
use led_core::theme::{Color as LedColor};

pub struct TabBar {
    workspace: Entity<Workspace>,
    scroll_offset: Pixels,
}

impl TabBar {
    pub fn new(workspace: Entity<Workspace>, _cx: &mut Context<Self>) -> Self {
        Self { 
            workspace,
            scroll_offset: px(0.0),
        }
    }

    fn led_color_to_gpui(&self, color: LedColor) -> Rgba {
        rgb((color.0 as u32) << 16 | (color.1 as u32) << 8 | (color.2 as u32))
    }

    fn handle_scroll(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let delta = event.delta.pixel_delta(px(1.0)).x;
        self.scroll_offset = (self.scroll_offset + delta).min(px(0.0));
        cx.notify();
    }
}

impl Render for TabBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let theme = &workspace.theme;
        let active_index = workspace.active_editor_index;

        div()
            .h_8()
            .w_full()
            .flex()
            .items_center()
            .bg(self.led_color_to_gpui(theme.ui.tab_bar_bg))
            .border_b_1()
            .border_color(self.led_color_to_gpui(theme.editor.line_number))
            .on_scroll_wheel(cx.listener(|this, event, _, cx| {
                this.handle_scroll(event, cx);
            }))
            .child(
                div()
                    .flex()
                    .h_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .h_full()
                            .ml(self.scroll_offset)
                            .children(
                                workspace.editors.iter().enumerate().map(|(idx, editor)| {
                                    let is_active = idx == active_index;
                                    let file_name = editor.path.as_ref()
                                        .and_then(|p| p.file_name())
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("[No Name]");
                                    let modified_flag = if editor.is_modified() { " [+]" } else { "" };
                                    let ro_flag = if editor.read_only { " [RO]" } else { "" };

                                    let bg_color = if is_active { 
                                        self.led_color_to_gpui(theme.ui.tab_active_bg) 
                                    } else { 
                                        self.led_color_to_gpui(theme.ui.tab_inactive_bg) 
                                    };
                                    let text_color = if is_active { 
                                        self.led_color_to_gpui(theme.ui.tab_active_fg) 
                                    } else { 
                                        self.led_color_to_gpui(theme.ui.tab_inactive_fg) 
                                    };

                                    div()
                                        .flex()
                                        .items_center()
                                        .px_3()
                                        .h_full()
                                        .bg(bg_color)
                                        .text_color(text_color)
                                        .border_r_1()
                                        .border_color(self.led_color_to_gpui(theme.editor.line_number))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .child(format!("{}{}{}", file_name, modified_flag, ro_flag))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    this.workspace.update(cx, |w, cx| {
                                                        w.active_editor_index = idx;
                                                        cx.notify();
                                                    });
                                                }))
                                        )
                                        .child(
                                            div()
                                                .ml_2()
                                                .child("×")
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    this.workspace.update(cx, |w, _| {
                                                        w.active_editor_index = idx;
                                                        w.close_active_editor();
                                                    });
                                                    cx.notify();
                                                }))
                                        )
                                        .on_mouse_down(MouseButton::Middle, cx.listener(move |this, _, _, cx| {
                                            this.workspace.update(cx, |w, _| {
                                                w.active_editor_index = idx;
                                                w.close_active_editor();
                                            });
                                            cx.notify();
                                        }))
                                }).collect::<Vec<_>>()
                            )
                    )
            )
    }
}
