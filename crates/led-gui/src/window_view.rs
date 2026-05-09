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

use crate::widgets::dialog::{Dialog, DialogType, DialogEvent};

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
    dialog: Option<Entity<Dialog>>,
    focus_handle: FocusHandle,
}

impl WindowView {
    pub fn new(config: Config, i18n: I18n, workspace: Entity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| EditorView::new(workspace.clone(), cx));
        let tab_bar = cx.new(|cx| TabBar::new(workspace.clone(), cx));
        let status_bar = cx.new(|cx| StatusBar::new(workspace.clone(), cx));
        let find_panel = cx.new(|cx| FindPanel::new(workspace.clone(), cx));
        #[cfg(not(target_os = "macos"))]
        let menu_bar = cx.new(|cx| MenuBar::new(workspace.clone(), i18n.clone(), cx));
        
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        Self {
            config,
            i18n,
            workspace,
            editor,
            tab_bar,
            status_bar,
            find_panel,
            #[cfg(not(target_os = "macos"))]
            menu_bar,
            dialog: None,
            focus_handle,
        }
    }

    fn show_dialog(&mut self, dialog_type: DialogType, window: &mut Window, cx: &mut Context<Self>) {
        let dialog = cx.new(|cx| Dialog::new(self.workspace.clone(), self.i18n.clone(), dialog_type, cx));
        cx.subscribe(&dialog, |this, _dialog, event, cx| {
            match event {
                DialogEvent::Close => {
                    this.dialog = None;
                    cx.notify();
                }
                DialogEvent::Save => {
                    this.workspace.update(cx, |w, _| {
                        let _ = w.active_editor_mut().save();
                    });
                    this.dialog = None;
                    cx.quit();
                }
                DialogEvent::DontSave => {
                    this.dialog = None;
                    cx.quit();
                }
                _ => {}
            }
        }).detach();
        self.dialog = Some(dialog.clone());
        dialog.update(cx, |d, cx| d.focus(window, cx));
        cx.notify();
    }

    fn handle_new(&mut self, _: &New, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            w.add_editor(Editor::new());
        });
        cx.notify();
    }

    fn handle_open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        self.show_dialog(DialogType::OpenFile, window, cx);
    }

    fn handle_save(&mut self, _: &Save, _window: &mut Window, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, _| {
            let editor = w.active_editor_mut();
            if editor.path.is_some() {
                let _ = editor.save();
            } else {
                // Should show Save As dialog, but we need window/cx here.
                // For now, let's assume handle_save_as is called elsewhere if path is None.
            }
        });
        cx.notify();
    }

    fn handle_save_as(&mut self, _: &SaveAs, window: &mut Window, cx: &mut Context<Self>) {
        self.show_dialog(DialogType::SaveAs, window, cx);
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

    fn handle_cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        let mut text_to_copy = None;
        self.workspace.update(cx, |w, _| {
            let editor = w.active_editor_mut();
            if let Some(range) = editor.selection.clone() {
                text_to_copy = Some(editor.rope.slice(range.clone()).to_string());
                editor.delete(range);
            }
        });
        if let Some(text) = text_to_copy {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        cx.notify();
    }

    fn handle_copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        let editor = workspace.active_editor();
        if let Some(range) = editor.selection.clone() {
            let text = editor.rope.slice(range).to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn handle_paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                let text = text.clone();
                self.workspace.update(cx, |w, _| {
                    let editor = w.active_editor_mut();
                    if let Some(range) = editor.selection.clone() {
                        editor.delete(range);
                    }
                    editor.insert(editor.cursor, &text);
                });
                cx.notify();
            }
        }
    }

    fn handle_find(&mut self, _: &Find, window: &mut Window, cx: &mut Context<Self>) {
        self.find_panel.update(cx, |p, cx| p.show(false, window, cx));
    }

    fn handle_replace(&mut self, _: &Replace, window: &mut Window, cx: &mut Context<Self>) {
        self.find_panel.update(cx, |p, cx| p.show(true, window, cx));
    }

    fn handle_toggle_line_numbers(&mut self, _: &ToggleLineNumbers, _window: &mut Window, cx: &mut Context<Self>) {
        self.config.line_numbers = !self.config.line_numbers;
        let _ = Config::write_key("line_numbers", &self.config.line_numbers.to_string());
        cx.notify();
    }

    fn handle_toggle_word_wrap(&mut self, _: &ToggleWordWrap, _window: &mut Window, cx: &mut Context<Self>) {
        self.config.word_wrap = !self.config.word_wrap;
        let _ = Config::write_key("word_wrap", &self.config.word_wrap.to_string());
        cx.notify();
    }

    fn handle_toggle_vi_mode(&mut self, _: &ToggleViMode, _window: &mut Window, cx: &mut Context<Self>) {
        self.config.vi_mode = !self.config.vi_mode;
        let _ = Config::write_key("vi_mode", &self.config.vi_mode.to_string());
        self.workspace.update(cx, |w, _| {
            for editor in w.editors.iter_mut() {
                editor.vi_mode = if self.config.vi_mode { led_core::ViMode::Normal } else { led_core::ViMode::Insert };
            }
        });
        cx.notify();
    }

    fn handle_go_to_line(&mut self, _: &GoToLine, window: &mut Window, cx: &mut Context<Self>) {
        self.show_dialog(DialogType::GoToLine, window, cx);
    }

    fn handle_about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        self.show_dialog(DialogType::About, window, cx);
    }

    fn handle_quit(&mut self, _: &Quit, window: &mut Window, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        let mut modified_file = None;
        for editor in &workspace.editors {
            if editor.is_modified() {
                modified_file = Some(editor.path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or(self.i18n.get("status.no_name").to_string()));
                break;
            }
        }

        if let Some(filename) = modified_file {
            self.show_dialog(DialogType::UnsavedChanges { filename }, window, cx);
        } else {
            cx.quit();
        }
    }

    fn handle_exit(&mut self, action: &Exit, window: &mut Window, cx: &mut Context<Self>) {
        self.handle_quit(&Quit {}, window, cx);
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
            .relative() // So dialog can be absolute
            .bg(bg)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::handle_new))
            .on_action(cx.listener(Self::handle_open))
            .on_action(cx.listener(Self::handle_save))
            .on_action(cx.listener(Self::handle_save_as))
            .on_action(cx.listener(Self::handle_close_tab))
            .on_action(cx.listener(Self::handle_next_tab))
            .on_action(cx.listener(Self::handle_prev_tab))
            .on_action(cx.listener(Self::handle_undo))
            .on_action(cx.listener(Self::handle_redo))
            .on_action(cx.listener(Self::handle_cut))
            .on_action(cx.listener(Self::handle_copy))
            .on_action(cx.listener(Self::handle_paste))
            .on_action(cx.listener(Self::handle_find))
            .on_action(cx.listener(Self::handle_replace))
            .on_action(cx.listener(Self::handle_toggle_line_numbers))
            .on_action(cx.listener(Self::handle_toggle_word_wrap))
            .on_action(cx.listener(Self::handle_toggle_vi_mode))
            .on_action(cx.listener(Self::handle_go_to_line))
            .on_action(cx.listener(Self::handle_about))
            .on_action(cx.listener(Self::handle_quit))
            .on_action(cx.listener(Self::handle_exit))
            .child(self.render_layout())
            .child(if let Some(ref dialog) = self.dialog {
                div().child(dialog.clone())
            } else {
                div()
            })
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
