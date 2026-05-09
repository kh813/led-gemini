use gpui::*;
use led_core::config::Config;
use led_core::i18n::I18n;
use crate::window_view::WindowView;
use crate::workspace::Workspace;

pub fn setup_app(app: &mut App) {
    let config = Config::load();
    let i18n = I18n::load(&config.language);

    // Platform-conditional menu setup
    setup_menu(app, &i18n);

    app.open_window(WindowOptions::default(), move |window, cx| {
        // Try cx.new for Model if new_model is not found
        let workspace = cx.new(|_| Workspace::new());
        cx.new(|cx| WindowView::new(config, i18n, workspace, window, cx))
    }).expect("Failed to open window");
}

#[cfg(target_os = "macos")]
fn setup_menu(app: &mut App, i18n: &I18n) {
    app.set_menus(build_native_menus(i18n));
}

#[cfg(not(target_os = "macos"))]
fn setup_menu(_app: &mut App, _i18n: &I18n) {
    // In-window menu bar is handled in window_view.rs
}

#[cfg(target_os = "macos")]
fn build_native_menus(i18n: &I18n) -> Vec<Menu> {
    vec![
        Menu {
            name: "led-gui".into(),
            items: vec![
                MenuItem::action("About led-gui", About {}),
                MenuItem::separator(),
                MenuItem::action("Quit led-gui", Quit {}),
            ],
            disabled: false,
        },
        Menu {
            name: i18n.get("menu.file").into(),
            items: vec![
                MenuItem::action(i18n.get("menu.file.new"), New {}),
                MenuItem::action(i18n.get("menu.file.open"), Open {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.file.save"), Save {}),
                MenuItem::action(i18n.get("menu.file.save_as"), SaveAs {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.file.close"), CloseTab {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.file.exit"), Exit {}),
            ],
            disabled: false,
        },
        Menu {
            name: i18n.get("menu.edit").into(),
            items: vec![
                MenuItem::action(i18n.get("menu.edit.undo"), Undo {}),
                MenuItem::action(i18n.get("menu.edit.redo"), Redo {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.edit.cut"), Cut {}),
                MenuItem::action(i18n.get("menu.edit.copy"), Copy {}),
                MenuItem::action(i18n.get("menu.edit.paste"), Paste {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.edit.find"), Find {}),
                MenuItem::action(i18n.get("menu.edit.replace"), Replace {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.edit.select_all"), SelectAll {}),
            ],
            disabled: false,
        },
        Menu {
            name: i18n.get("menu.view").into(),
            items: vec![
                MenuItem::action(i18n.get("menu.view.go_to_line"), GoToLine {}),
                MenuItem::separator(),
                MenuItem::action(i18n.get("menu.view.line_numbers"), ToggleLineNumbers {}),
                MenuItem::action(i18n.get("menu.view.word_wrap"), ToggleWordWrap {}),
                MenuItem::action(i18n.get("menu.view.vi_mode"), ToggleViMode {}),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: i18n.get("menu.view.encoding").into(),
                    items: vec![
                        MenuItem::action("UTF-8", NoOp {}),
                        MenuItem::action("Shift-JIS", NoOp {}),
                    ],
                    disabled: false,
                }),
                MenuItem::submenu(Menu {
                    name: i18n.get("menu.view.line_ending").into(),
                    items: vec![
                        MenuItem::action("LF", NoOp {}),
                        MenuItem::action("CRLF", NoOp {}),
                    ],
                    disabled: false,
                }),
                MenuItem::submenu(Menu {
                    name: i18n.get("menu.view.theme").into(),
                    items: vec![
                        MenuItem::action("Tokyo Night", NoOp {}),
                        MenuItem::action("Solarized Dark", NoOp {}),
                    ],
                    disabled: false,
                }),
            ],
            disabled: false,
        },
        Menu {
            name: "Tabs".into(),
            items: vec![
                MenuItem::action("Next Tab", NextTab {}),
                MenuItem::action("Previous Tab", PrevTab {}),
            ],
            disabled: false,
        },
        Menu {
            name: i18n.get("menu.help").into(),
            items: vec![
                MenuItem::action(i18n.get("menu.help.about"), About {}),
            ],
            disabled: false,
        }
    ]
}

actions!(led, [
    // App/File
    About, Quit, Exit, New, Open, Save, SaveAs, CloseTab,
    // Edit
    Undo, Redo, Cut, Copy, Paste, Find, Replace, SelectAll,
    // Tabs
    NextTab, PrevTab,
    // View
    GoToLine, ToggleLineNumbers, ToggleWordWrap, ToggleViMode,
    // Search
    SearchNext, SearchPrev, SearchReplace, SearchReplaceAll,
    ToggleMatchCase, ToggleWholeWord, ToggleRegex,
    // Other
    NoOp
]);
