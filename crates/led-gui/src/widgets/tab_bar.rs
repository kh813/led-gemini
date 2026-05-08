use gpui::*;

pub struct TabBar {}

impl TabBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for TabBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_8()
            .w_full()
            .flex()
            .items_center()
            .bg(rgb(0x16161e))
            .border_b_1()
            .border_color(rgb(0x3b4261))
            .child(div().px_2().child("Tab Bar Stub"))
    }
}
