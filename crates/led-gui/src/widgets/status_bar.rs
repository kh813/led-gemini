use gpui::*;

pub struct StatusBar {}

impl StatusBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_6()
            .w_full()
            .flex()
            .items_center()
            .bg(rgb(0x16161e))
            .border_t_1()
            .border_color(rgb(0x3b4261))
            .child(div().px_2().child("Status Bar Stub"))
    }
}
