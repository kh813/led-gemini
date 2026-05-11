pub mod menu_bar;
pub mod editor_view;
pub mod tab_bar;
pub mod find_panel;
pub mod status_bar;
pub mod dialog;

use gpui::Rgba;
use led_core::theme::Color as LedColor;

pub fn led_color_to_gpui(color: LedColor) -> Rgba {
    Rgba {
        r: color.0 as f32 / 255.0,
        g: color.1 as f32 / 255.0,
        b: color.2 as f32 / 255.0,
        a: 1.0,
    }
}
