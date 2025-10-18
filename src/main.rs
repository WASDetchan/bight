use bight::{
    editor::{Editor, bindings::Callback, view::EditorView},
    key::parse_key_sequence,
    mode::Mode,
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

    let view = EditorView::new(editor).full_screen();

    cursive.add_fullscreen_layer(view);
    cursive.run();
}
