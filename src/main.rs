use bight::{
    editor::{Editor, bindings::Callback, view::EditorView},
    key::parse_key_sequence,
    mode::Mode,
    table::{Table, cell::CellContent},
};
use cursive::{Cursive, CursiveExt, event::Key, view::Resizable};

fn main() {
    let mut cursive = Cursive::default();

    let mut editor = Editor::default();

    let esc_seq = vec![Key::Esc.into()];

    editor.add_callback_binding(
        &Mode::Normal,
        &parse_key_sequence("q").unwrap(),
        Callback::with_cursive(|s| s.quit()),
    );
    editor.add_callback_binding(
        &Mode::Insert,
        &esc_seq,
        Callback::with_state(|state| state.mode = Mode::Normal),
    );
    editor
        .add_callback_bindings_str(
            "n",
            "i",
            Callback::with_state(|state| state.mode = Mode::Insert),
        )
        .unwrap();

    add_value_callbacks(&mut editor);
    add_move_callbacks(&mut editor);

    let view = EditorView::new(editor).full_screen();

    cursive.add_fullscreen_layer(view);
    cursive.run();
}

fn add_value_callbacks(editor: &mut Editor) {
    editor
        .add_callback_bindings_str(
            "n",
            "p",
            Callback::with_state(|state| {
                let pos = state.cursor;
                let v = state.table.get(pos);
                let v = if let Some(CellContent::Value(v)) = v {
                    *v
                } else {
                    0
                };

                eprintln!("{v}");
                state.table.set(pos, Some(CellContent::Value(v + 1)));
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "o",
            Callback::with_state(|state| {
                let pos = state.cursor;
                let v = state.table.get(pos);
                let v = if let Some(CellContent::Value(v)) = v {
                    *v
                } else {
                    0
                };

                eprintln!("{v}");
                state.table.set(pos, Some(CellContent::Value(v - 1)));
            }),
        )
        .unwrap();
}

fn add_move_callbacks(editor: &mut Editor) {
    editor
        .add_callback_bindings_str(
            "n",
            "l",
            Callback::with_state(|state| {
                state.cursor.x = state.cursor.x.saturating_add(1);
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "h",
            Callback::with_state(|state| {
                state.cursor.x = state.cursor.x.saturating_sub(1);
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "j",
            Callback::with_state(|state| {
                state.cursor.y = state.cursor.y.saturating_add(1);
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "k",
            Callback::with_state(|state| {
                state.cursor.y = state.cursor.y.saturating_sub(1);
            }),
        )
        .unwrap();
}
