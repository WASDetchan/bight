use cursive::{
    View,
    event::{Event, EventResult},
    view::ViewWrapper,
    views::{LinearLayout, TextView},
};

use crate::{editor::{bindings::Callback, Editor}, key::Key};

pub struct EditorView {
    editor: Editor,
    layout: LinearLayout,
}

impl EditorView {
    pub fn new(editor: Editor) -> Self {
        Self {
            editor,
            layout: LinearLayout::vertical(),
        }
    }

    pub fn upadate_layout(&mut self) {
        let spreadsheet = TextView::new(format!("{:?}", self.editor));
        let status_bar = self.status_bar();

        self.layout = LinearLayout::vertical()
            .child(spreadsheet)
            .child(status_bar);
    }

    pub fn status_bar(&self) -> impl View {
        let status = self.editor.display_mode();
        TextView::new(status)
    }
}

impl ViewWrapper for EditorView {
    cursive::wrap_impl!(self.layout: LinearLayout);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        let editor = &mut self.editor;
        editor.key_sequence.push(Key(event));
        if let Some(cb) : Option<Callback> = editor.handle_sequence() {
cb.state(editor.state.write().unwrap());
            EventResult::with_cb(move |s| {
                (cb.cursive)(s);
            })
        } else {
            EventResult::consumed()
        }
    }
}
