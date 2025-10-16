use bight::{
    editor::{Editor, EditorCommand, EditorView},
    key::parse_key_sequence,
    mode::Mode,
};
use cursive::{Cursive, CursiveExt, Vec2, view::Resizable};

fn main() {
    let mut cursive = Cursive::default();

    let mut editor = Editor::default();
    // let bindings = KeyBindings {
    //     insert: 'i'.into(),
    //     normal: Key::Esc.into(),
    //     quit: 'q'.into(),
    // };
    // editor.add_bindings(bindings);
    //
    //
    // add_insert_callback(&mut editor);
    // add_normal_callback(&mut editor);
    // add_quit_callback(&mut editor);

    let esc_seq = vec![bight::key::Key(cursive::event::Key::Esc.into())];

    editor.add_command_binding(
        Mode::Normal,
        &parse_key_sequence("q").unwrap(),
        EditorCommand::Quit,
    );
    editor.add_command_binding(Mode::Insert, &esc_seq, EditorCommand::NormalMode);
    editor.add_command_bindings_str(
        "n",
        "i",
        EditorCommand::InsertMode,
    ).unwrap();

    editor.add_command_binding(
        Mode::Normal,
        &parse_key_sequence("aaaba").unwrap(),
        EditorCommand::InsertMode,
    );

    editor.add_command_binding(
        Mode::Normal,
        &parse_key_sequence("aaabc").unwrap(),
        EditorCommand::InsertMode,
    );

    let view = EditorView::new(editor).fixed_size(Vec2::new(100, 100));

    cursive.add_layer(view);
    cursive.run();
}
