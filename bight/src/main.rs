use std::io::{Write, stdout};

use bight::{
    app::AppState,
    callback::{EditorStateCallback, OnKeyEventCallback as CB},
    editor::{
        EditorState,
        bindings::{
            EditorBindings,
            vim_default::{add_clipboard_binding, add_mode_bindings, add_move_callbacks},
        },
    },
    key::Key,
    table::slice::table::TableSlice,
    term::view::{DrawRect, editor},
};
use crossterm::terminal::{self, ClearType};
use edit::Builder;

fn main() {
    env_logger::init();

    let mut editor = EditorState::default();
    let mut app = AppState { run: true };

    let mut bindings = EditorBindings::default();

    add_clipboard_binding(&mut bindings);
    add_value_callbacks(&mut bindings);
    add_move_callbacks(&mut bindings);
    add_mode_bindings(&mut bindings);

    let mut sequence = Vec::new();
    let mut stdout = stdout();

    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();

    draw(&editor, &sequence);
    while app.run {
        let event = crossterm::event::read().expect("idk what error can occur here");
        let Ok(key) = event.try_into() else {
            continue;
        };
        sequence.push(key);
        if let Some(cb) = bindings.handle_sequence(&mut sequence, editor.mode) {
            match cb {
                CB::EditorStateChanage(cb) => (cb.0)(&mut editor),
                CB::AppStateChange(cb) => (cb.0)(&mut app),
            }

            editor.table.evaluate();
        }

        draw(&editor, &sequence);
    }

    terminal::disable_raw_mode().unwrap();
    crossterm::execute!(
        stdout,
        terminal::Clear(ClearType::All),
        crossterm::terminal::LeaveAlternateScreen
    )
    .unwrap();
}

fn draw(editor: &EditorState, sequence: &[Key]) {
    let mut stdout = stdout();
    let data = TableSlice::new(((0, 0), (50, 50)), &editor.table);
    let rect = DrawRect::full_term();
    editor::draw(&mut stdout, rect, editor, sequence, data);
    stdout.flush().unwrap();
}

fn add_value_callbacks(editor: &mut EditorBindings) {
    editor
        .add_callback_bindings_str(
            "n",
            "I",
            EditorStateCallback::new(|state| {
                let pos = state.cursor;
                let v = state.table.get_source(pos);
                let v: &str = if let Some(v) = v { v } else { "" };
                let mut builder = Builder::new();
                builder.suffix(".bcell");
                let new_source = edit::edit_with_builder(v, &builder).unwrap();
                state.table.set_source(pos, Some(new_source));
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "K",
            EditorStateCallback::new(|state| state.expand = !state.expand),
        )
        .unwrap();
}
