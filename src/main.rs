use std::io::{Write, stdout};

use bight::{
    app::AppState, callback::{EditorStateCallback, OnKeyEventCallback as CB}, editor::{
        bindings::{
            vim_default::{add_mode_bindings, add_move_callbacks}, EditorBindings
        }, EditorState
    }, key::Key, mode::Mode, table::{
        cell::CellContent, slice::table::TableSlice, Table
    }, term::view::{editor, DrawRect}
};
use crossterm::terminal::{self, ClearType};

fn main() {
    let mut editor = EditorState::default();
    let mut app = AppState { run: true };

    let mut bindings = EditorBindings::default();

    add_value_callbacks(&mut bindings);
    add_move_callbacks(&mut bindings);
    add_mode_bindings(&mut bindings);

    bindings
        .add_callback_bindings_str(
            "n",
            "abcde",
            EditorStateCallback::new(|state| state.mode = Mode::Insert),
        )
        .unwrap();

    bindings
        .add_callback_bindings_str(
            "n",
            "abCde",
            EditorStateCallback::new(|state| state.mode = Mode::Insert),
        )
        .unwrap();
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
            "p",
            EditorStateCallback::new(|state| {
                let pos = state.cursor;
                let v = state.table.get(pos);
                let v = if let Some(CellContent::Value(v)) = v {
                    *v
                } else {
                    0
                };

                state.table.set(pos, Some(CellContent::Value(v + 1)));
            }),
        )
        .unwrap();
    editor
        .add_callback_bindings_str(
            "n",
            "o",
            EditorStateCallback::new(|state| {
                let pos = state.cursor;
                let v = state.table.get(pos);
                let v = if let Some(CellContent::Value(v)) = v {
                    *v
                } else {
                    0
                };

                state.table.set(pos, Some(CellContent::Value(v - 1)));
            }),
        )
        .unwrap();
}
