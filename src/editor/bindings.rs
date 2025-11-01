pub mod vim_default {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::{
        callback::{AppStateCallback, EditorStateCallback},
        key::parse_key_sequence,
        mode::Mode,
    };

    use super::EditorBindings;
    pub fn add_mode_bindings(bindings: &mut EditorBindings) {
        let esc_seq = vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE).into()];

        bindings.add_callback_binding(
            &Mode::Normal,
            &parse_key_sequence("q").unwrap(),
            AppStateCallback::new(|state| state.run = false),
        );
        bindings.add_callback_binding(
            &Mode::Insert,
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
}

use crate::{
    callback::{KeyBindTree, OnKeyEventCallback as Callback},
    key::{Key, KeySequenceError, SequenceParseError, parse_key_sequence},
    mode::{Mode, ModeParseError, parse_modes},
};

#[derive(Default)]
pub struct EditorBindings {
    pub normal: KeyBindTree,
    pub insert: KeyBindTree,
}

#[derive(Debug, thiserror::Error)]
pub enum BindingParseError {
    #[error(transparent)]
    KeySequenceParseError(#[from] SequenceParseError),
    #[error(transparent)]
    ModeParseError(#[from] ModeParseError),
}

impl EditorBindings {
    pub fn handle_sequence(&self, sequence: &mut Vec<Key>, mode: Mode) -> Option<Callback> {
        let tree = match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
            Mode::Cell => todo!(),
        };
        let cb = loop {
            let cb = tree.find(sequence);
            if cb.is_ok()
                || cb.is_err_and(|e| e != KeySequenceError::InvalidSequence)
                || sequence.is_empty()
            {
                break cb;
            }
            sequence.remove(0);
        };
        match cb {
            Ok(cb) => {
                sequence.clear();

                cb.get(0).cloned() // TODO: Actually support multiple cbs per binding
            }
            Err(_) => None,
        }
    }

    pub fn add_callback_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        cb: impl Into<Callback>,
    ) -> Result<(), BindingParseError> {
        let cb = cb.into();
        let sequence = parse_key_sequence(sequence)?;
        let modes = parse_modes(modes)?;

        for mode in modes {
            self.add_callback_binding(&mode, &sequence, cb.clone());
        }

        Ok(())
    }

    pub fn add_callback_binding(&mut self, mode: &Mode, sequence: &[Key], cb: impl Into<Callback>) {
        let cb = cb.into();
        match mode {
            Mode::Normal => self.normal.map(sequence, Some(vec![cb])),
            Mode::Insert => self.insert.map(sequence, Some(vec![cb])),
            Mode::Cell => todo!(),
        }
    }
}
