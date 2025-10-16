use cursive::{
    View,
    event::{Event, EventResult},
    view::{Resizable, ViewWrapper},
    views::{LinearLayout, TextView},
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
        let status =format!( "{} {}", self.editor.display_mode(), self.editor.display_sequence());
        TextView::new(status)
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
