use cursive::{
    View,
    event::{Event, EventResult},
    view::{Resizable, ViewWrapper},
    views::{DummyView, LinearLayout, TextView},
};

use crate::{
    editor::Editor,
    key::Key,
    table::{
        slice::table::TableSlice,
        view::{TableStyle, TableView},
    },
    view::grid::GridLayout,
};

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

    fn init_layout(&mut self) {
        let editor = self.editor.state.read().unwrap();
        let table_slice = TableSlice::new(((0, 0), (10, 15)), &editor.table);

        let style = TableStyle {
            cell_width: 10,
            ..Default::default()
        };

        let spreadsheet = TableView::from_table_slice(table_slice, style).full_screen();

        let mut spreadsheet = GridLayout::from_2d_vec(vec![
            vec![TextView::new("hello,"), TextView::new("world")],
            vec![TextView::new("hiiiii"), TextView::new("")],
        ])
        .unwrap();

        spreadsheet.replace(1, 1, TextView::new(" everyone"));

        let status_bar = self.status_bar();

        self.layout = LinearLayout::vertical()
            .child(spreadsheet)
            .child(status_bar);
    }

    fn upadate_layout(&mut self) {
        self.init_layout();
    }

    pub fn status_bar_mode(&self) -> String {
        format!(" {} ", self.editor.display_mode())
    }
    pub fn status_bar_sequence(&self) -> String {
        format!(" { } ", self.editor.display_sequence())
    }
    pub fn status_bar(&self) -> impl View {
        let status_left = self.status_bar_mode();
        let status_right = self.status_bar_sequence();
        LinearLayout::horizontal()
            .child(TextView::new(status_left))
            .child(DummyView.full_width())
            .child(TextView::new(status_right))
            .full_width()
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
