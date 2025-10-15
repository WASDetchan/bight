use bight::{
    callback::AddModeCallback,
    editor::{Editor, KeyBindings},
    mode::{HasMode, Mode},
};
use cursive::event::Key;

fn add_insert_callback(editor: &mut Editor) {
    let mode = editor.get_mode();
    editor.add_non_text_callback('i', move |_| *mode.write().unwrap() = Mode::Insert);
}
fn add_normal_callback(editor: &mut Editor) {
    let mode = editor.get_mode();
    editor.add_callback(Key::Esc, move |_| *mode.write().unwrap() = Mode::Normal);
}

fn add_quit_callback(editor: &mut Editor) {
    editor.add_non_text_callback('q', |s| s.quit());
}

fn main() {
    let mut editor = Editor::default();
    let bindings = KeyBindings {
        insert: 'i'.into(),
        normal: Key::Esc.into(),
        quit: 'q'.into(),
    };
    editor.add_bindings(bindings);
    // add_insert_callback(&mut editor);
    // add_normal_callback(&mut editor);
    // add_quit_callback(&mut editor);
    editor.run();
}
