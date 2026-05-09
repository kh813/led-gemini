use gpui::*;
use crate::workspace::Workspace;
use led_core::i18n::I18n;

pub struct MenuBar {
    workspace: Entity<Workspace>,
    i18n: I18n,
    open_menu: Option<usize>,
}

impl MenuBar {
    pub fn new(workspace: Entity<Workspace>, i18n: I18n, _cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            i18n,
            open_menu: None,
        }
    }

    fn toggle_menu(&mut self, idx: usize, _window: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu == Some(idx) {
            self.open_menu = None;
        } else {
            self.open_menu = Some(idx);
        }
        cx.notify();
    }
}

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let theme = &workspace.theme;
        let bg = rgb((theme.ui.panel_bg.0 as u32) << 16 | (theme.ui.panel_bg.1 as u32) << 8 | (theme.ui.panel_bg.2 as u32));
        let fg = rgb((theme.ui.panel_fg.0 as u32) << 16 | (theme.ui.panel_fg.1 as u32) << 8 | (theme.ui.panel_fg.2 as u32));
        let border = rgb((theme.ui.dialog_border.0 as u32) << 16 | (theme.ui.dialog_border.1 as u32) << 8 | (theme.ui.dialog_border.2 as u32));

        div()
            .w_full()
            .h(px(24.0))
            .bg(bg)
            .text_color(fg)
            .border_b_1()
            .border_color(border)
            .flex()
            .items_center()
            .px_2()
            .gap_4()
            .child(
                div()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.toggle_menu(0, window, cx)))
                    .child(self.i18n.get("menu.file").to_string())
            )
            .child(
                div()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.toggle_menu(1, window, cx)))
                    .child(self.i18n.get("menu.edit").to_string())
            )
            .child(
                div()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.toggle_menu(2, window, cx)))
                    .child(self.i18n.get("menu.view").to_string())
            )
            .child(
                div()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.toggle_menu(3, window, cx)))
                    .child(self.i18n.get("menu.help").to_string())
            )
            // Dropdowns (Simplified: just a placeholder for now, as proper overlays are complex)
            .child(if let Some(idx) = self.open_menu {
                div()
                    .absolute()
                    .top(px(24.0))
                    .left(px(idx as f32 * 50.0 + 8.0)) // Rough position
                    .w(px(150.0))
                    .bg(bg)
                    .border_1()
                    .border_color(border)
                    .shadow_md()
                    .child(format!("Menu {} Dropdown", idx))
            } else {
                div()
            })
    }
}
