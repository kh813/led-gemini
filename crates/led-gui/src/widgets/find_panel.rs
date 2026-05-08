use gpui::*;

pub struct FindPanel {}

impl FindPanel {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for FindPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_0()
            .w_full()
            .bg(rgb(0x16161e))
            .child("Find Panel Stub")
    }
}
