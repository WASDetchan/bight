use std::io::stdout;

use bight::{
    app::AppState,
    callback::{EditorStateCallback, OnKeyEventCallback as CB},
    editor::{
        EditorState,
        bindings::{
            EditorBindings,
            vim_default::{add_mode_bindings, add_move_callbacks},
        },
        display_sequence,
    },
    table::{Table, cell::CellContent},
};
use crossterm::{
    cursor, style,
    terminal::{self, ClearType},
};

fn main() {
    let mut editor = EditorState::default();
    let mut app = AppState { run: true };

    let mut bindings = EditorBindings::default();

    add_value_callbacks(&mut bindings);
    add_move_callbacks(&mut bindings);
    add_mode_bindings(&mut bindings);

    let mut sequence = Vec::new();
    let mut stdout = stdout();

    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
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
        crossterm::execute!(stdout,).unwrap();
        crossterm::execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            style::Print(format!("current sequence: {}", display_sequence(&sequence)))
        )
        .unwrap();
    }

    terminal::disable_raw_mode().unwrap();
    crossterm::execute!(
        stdout,
        terminal::Clear(ClearType::All),
        crossterm::terminal::LeaveAlternateScreen
    )
    .unwrap();
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

                eprintln!("{v}");
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

                eprintln!("{v}");
                state.table.set(pos, Some(CellContent::Value(v - 1)));
            }),
        )
        .unwrap();
}
