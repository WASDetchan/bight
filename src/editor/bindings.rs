use std::sync::Arc;

use cursive::Cursive;

use crate::{
    editor::State,
    key::{Key, KeySequenceError, KeyTree, SequenceParseError, parse_key_sequence},
    mode::{Mode, ModeParseError, parse_modes},
};

#[derive(Clone)]
pub struct Callback {
    pub state: Arc<dyn Fn(&mut State) + Send + Sync + 'static>,
    pub cursive: Arc<dyn Fn(&mut Cursive) + Send + Sync + 'static>,
}

impl Callback {
    pub fn empty() -> Self {
        Self {
            state: Arc::new(|_| {}),
            cursive: Arc::new(|_| {}),
        }
    }

    pub fn with_state(f: impl Fn(&mut State) + Send + Sync + 'static) -> Self {
        Self {
            state: Arc::new(f),
            cursive: Arc::new(|_| {}),
        }
    }

    pub fn with_cursive(f: impl Fn(&mut Cursive) + Send + Sync + 'static) -> Self {
        Self {
            state: Arc::new(|_| {}),
            cursive: Arc::new(f),
        }
    }
}

pub type KeyBindTree = KeyTree<Callback>;

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
    pub fn hadndle_sequence(&self, sequence: &mut Vec<Key>, mode: &Mode) -> Option<Callback> {
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

                Some(cb.clone())
            }
            Err(_) => None,
        }
    }

    pub fn add_callback_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        cb: Callback,
    ) -> Result<(), BindingParseError> {
        let sequence = parse_key_sequence(sequence)?;
        let modes = parse_modes(modes)?;

        for mode in modes {
            self.add_callback_binding(&mode, &sequence, cb.clone());
        }

        Ok(())
    }

    pub fn add_callback_binding(&mut self, mode: &Mode, sequence: &[Key], cb: Callback) {
        match mode {
            Mode::Normal => self.normal.map(sequence, Some(cb)),
            Mode::Insert => self.insert.map(sequence, Some(cb)),
            Mode::Cell => todo!(),
        }
    }
}
