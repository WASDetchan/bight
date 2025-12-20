use std::
    sync::Arc
;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    callback::{AppStateCallback, EditorStateCallback},
    clipboard::{get_clipboard, set_clipboard},
    mode::Mode,
    sequence::parse_key_sequence,
};

use super::EditorBindings;


pub fn add_clipboard_binding(bindings: &mut EditorBindings) {
    bindings
        .add_callback_bindings_str(
            "n",
            "yy",
            EditorStateCallback::new(|state| {
                let pos = state.cursor;
                let v = state.table.get_source(pos);
                let v: Arc<str> = if let Some(v) = v {
                    v.clone()
                } else {
                    Arc::from("")
                };
                set_clipboard(v);
            }),
        )
        .unwrap();
    bindings
        .add_callback_bindings_str(
            "n",
            "p",
            EditorStateCallback::new(|state| {
                let pos = state.cursor;
                if let Some(v) = get_clipboard() {
                    state.table.set_source(pos, Some(v));
                }
            }),
        )
        .unwrap();
}

pub fn add_mode_bindings(bindings: &mut EditorBindings) {
    let esc_seq = vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE).into()];

    bindings.add_callback_binding(
        Mode::Normal,
        &parse_key_sequence("q").unwrap(),
        AppStateCallback::new(|state| state.run = false),
    );
    bindings.add_callback_binding(
        Mode::Insert,
        &esc_seq,
        EditorStateCallback::new(|state| state.mode = Mode::Normal),
    );
    bindings
        .add_callback_bindings_str(
            "n",
            "i",
            EditorStateCallback::new(|state| state.mode = Mode::Insert),
        )
        .unwrap();
}
pub fn add_move_callbacks(bindings: &mut EditorBindings) {
    bindings
        .add_callback_bindings_str(
            "n",
            "l",
            EditorStateCallback::new(|state| {
                state.cursor.x = state.cursor.x.saturating_add(1);
            }),
        )
        .unwrap();
    bindings
        .add_callback_bindings_str(
            "n",
            "h",
            EditorStateCallback::new(|state| {
                state.cursor.x = state.cursor.x.saturating_sub(1);
            }),
        )
        .unwrap();
    bindings
        .add_callback_bindings_str(
            "n",
            "j",
            EditorStateCallback::new(|state| {
                state.cursor.y = state.cursor.y.saturating_add(1);
            }),
        )
        .unwrap();
    bindings
        .add_callback_bindings_str(
            "n",
            "k",
            EditorStateCallback::new(|state| {
                state.cursor.y = state.cursor.y.saturating_sub(1);
            }),
        )
        .unwrap();
}
