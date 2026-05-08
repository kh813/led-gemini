use gpui::*;

mod app;
mod window_view;
mod widgets;
mod workspace;

use crate::app::setup_app;

fn main() {
    gpui_platform::application().run(|app: &mut App| {
        setup_app(app);
    });
}
