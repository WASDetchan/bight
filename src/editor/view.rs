use cursive::{
    event::{Event, EventResult}, view::{Resizable, ViewWrapper}, views::{DummyView, LinearLayout, TextView}, View
};

use crate::{editor::Editor, key::Key};

pub struct EditorView {
    editor: Editor,
    layout: LinearLayout,
}

impl EditorView {
    pub fn new(editor: Editor) -> Self {
        let mut s = Self {
            editor,
            layout: LinearLayout::vertical(),
        };
        s.upadate_layout();
        s
    }

    pub fn upadate_layout(&mut self) {
        let spreadsheet = TextView::new(format!("{:?}", self.editor)).full_screen();
        let status_bar = self.status_bar();

        self.layout = LinearLayout::vertical()
            .child(spreadsheet)
            .child(status_bar);
    }

    pub fn status_bar(&self) -> impl View {
        let status_left =format!(" {} ", self.editor.display_mode());
        let status_right = format!(" { } ", self.editor.display_sequence());
         LinearLayout::horizontal().child(TextView::new(status_left)).child(DummyView.full_width()).child(TextView::new(status_right)).full_width()

    }
}

impl ViewWrapper for EditorView {
    cursive::wrap_impl!(self.layout: LinearLayout);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        let editor = &mut self.editor;
        editor.key_sequence.push(Key(event));
        if let Some(cb) = editor.handle_sequence() {
            (cb.state)(&mut editor.state.write().unwrap());
            self.upadate_layout();
            EventResult::with_cb(move |s| {
                (cb.cursive)(s);
            })
        } else {
            self.upadate_layout();
            EventResult::consumed()
        }
    }
}
