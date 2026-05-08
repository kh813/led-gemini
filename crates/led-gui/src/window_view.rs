use gpui::*;
use led_core::config::Config;
use led_core::i18n::I18n;
use crate::widgets::editor_view::EditorView;
use crate::widgets::tab_bar::TabBar;
use crate::widgets::status_bar::StatusBar;
use crate::widgets::find_panel::FindPanel;
use crate::workspace::Workspace;
use crate::app::*;
use led_core::buffer::Editor;

#[cfg(not(target_os = "macos"))]
use crate::widgets::menu_bar::MenuBar;

pub struct WindowView {
    config: Config,
    i18n: I18n,
    workspace: Entity<Workspace>,
    editor: Entity<EditorView>,
    tab_bar: Entity<TabBar>,
    status_bar: Entity<StatusBar>,
    find_panel: Entity<FindPanel>,
    #[cfg(not(target_os = "macos"))]
    menu_bar: Entity<MenuBar>,
}

impl WindowView {
    pub fn new(config: Config, i18n: I18n, workspace: Entity<Workspace>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            config: config.clone(),
            i18n: i18n.clone(),
            workspace: workspace.clone(),
            editor: cx.new(|cx| EditorView::new(workspace.clone(), cx)),
            tab_bar: cx.new(|cx| TabBar::new(workspace.clone(), cx)),
            status_bar: cx.new(|cx| StatusBar::new(workspace.clone(), cx)),
            find_panel: cx.new(|cx| FindPanel::new(cx)),
            #[cfg(not(target_os = "macos"))]
            menu_bar: cx.new(|cx| MenuBar::new(cx)),
        }
    }

    fn handle_new(&mut self, _: &New, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.add_editor(Editor::new());
        });
        cx.notify();
    }

    fn handle_close_tab(&mut self, _: &CloseTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.close_active_editor();
        });
        cx.notify();
    }

    fn handle_next_tab(&mut self, _: &NextTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.next_tab();
        });
        cx.notify();
    }

    fn handle_prev_tab(&mut self, _: &PrevTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.prev_tab();
        });
        cx.notify();
    }

    fn handle_undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.active_editor_mut().undo();
        });
        cx.notify();
    }

    fn handle_redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.active_editor_mut().redo();
        });
        cx.notify();
    }
}

impl Render for WindowView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let theme = &workspace.theme;
        let bg = rgb((theme.editor.background.0 as u32) << 16 | (theme.editor.background.1 as u32) << 8 | (theme.editor.background.2 as u32));

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .bg(bg)
            .on_action(cx.listener(Self::handle_new))
            .on_action(cx.listener(Self::handle_close_tab))
            .on_action(cx.listener(Self::handle_next_tab))
            .on_action(cx.listener(Self::handle_prev_tab))
            .on_action(cx.listener(Self::handle_undo))
            .on_action(cx.listener(Self::handle_redo))
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
