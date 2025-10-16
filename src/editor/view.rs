use cursive::{
    View,
    event::{Event, EventResult},
};

use crate::{editor::Editor, key::Key};

pub struct EditorView {
    editor: Editor,
}

impl EditorView {
    pub fn new(editor: Editor) -> Self {
        Self { editor }
    }
}

impl View for EditorView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &format!("{:?}", self.editor));
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let editor = &mut self.editor;
        editor.key_sequence.push(Key(event));
        if let Some(cb) = editor.handle_sequence() {
            EventResult::with_cb(move |s| (cb)(s))
        } else {
            EventResult::consumed()
        }
    }
}
