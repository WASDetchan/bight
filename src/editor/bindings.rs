use std::sync::Arc;

use cursive::Cursive;

use crate::{
    key::{Key, KeySequenceError, KeyTree, SequenceParseError, parse_key_sequence},
    mode::{Mode, ModeParseError, parse_modes},
};

pub type Callback = Arc<dyn Fn(&mut Cursive) + Send + Sync + 'static>;
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

                Some(Arc::clone(cb))
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
