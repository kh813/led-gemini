use gpui::*;

pub struct EditorView {}

impl EditorView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(rgb(0x3b4261))
            .child("Editor View Stub")
    }
}
