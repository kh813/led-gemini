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
            ],
            disabled: false,
        },
        Menu {
            name: i18n.get("menu.edit").into(),
            items: vec![
                MenuItem::action(i18n.get("menu.edit.undo"), Undo {}),
                MenuItem::action(i18n.get("menu.edit.redo"), Redo {}),
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
        }
    ]
}

actions!(led, [About, Quit, New, Open, Save, SaveAs, CloseTab, NextTab, PrevTab, Undo, Redo]);
