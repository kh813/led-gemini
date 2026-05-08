use gpui::*;
use led_core::config::Config;
use led_core::i18n::I18n;
use crate::widgets::editor_view::EditorView;
use crate::widgets::tab_bar::TabBar;
use crate::widgets::status_bar::StatusBar;
use crate::widgets::find_panel::FindPanel;

#[cfg(not(target_os = "macos"))]
use crate::widgets::menu_bar::MenuBar;

pub struct WindowView {
    config: Config,
    i18n: I18n,
    editor: Entity<EditorView>,
    tab_bar: Entity<TabBar>,
    status_bar: Entity<StatusBar>,
    find_panel: Entity<FindPanel>,
    #[cfg(not(target_os = "macos"))]
    menu_bar: Entity<MenuBar>,
}

impl WindowView {
    pub fn new(config: Config, i18n: I18n, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            config: config.clone(),
            i18n: i18n.clone(),
            editor: cx.new(|cx| EditorView::new(cx)),
            tab_bar: cx.new(|cx| TabBar::new(cx)),
            status_bar: cx.new(|cx| StatusBar::new(cx)),
            find_panel: cx.new(|cx| FindPanel::new(cx)),
            #[cfg(not(target_os = "macos"))]
            menu_bar: cx.new(|cx| MenuBar::new(cx)),
        }
    }
}

impl Render for WindowView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1a1b26)) // Tokyo Night background
            .text_color(rgb(0xc0caf5))
            .child(self.render_layout())
    }
}

impl WindowView {
    fn render_layout(&self) -> impl IntoElement {
        let container = div().w_full().h_full().flex().flex_col();

        #[cfg(not(target_os = "macos"))]
        let container = container.child(self.menu_bar.clone());

        container
            .child(self.tab_bar.clone())
            .child(self.find_panel.clone())
            .child(div().flex_grow().child(self.editor.clone()))
            .child(self.status_bar.clone())
    }
}
