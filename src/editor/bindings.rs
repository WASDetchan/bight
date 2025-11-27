pub mod vim_default;

use std::{
    collections::{HashMap, HashSet},
    process::Output,
    sync::Arc,
};

use mlua::Error;

use crate::{
    callback::OnKeyEventCallback as Callback,
    key::{
        Key,
        sequence::{
            MatchKeySequence, SequenceBinding, SequenceBindingError, SequenceParseError,
            parse_key_sequence,
        },
    },
    mode::{Mode, ModeParseError, parse_modes},
};
type CallbackSequenceBinding =
    Box<dyn MatchKeySequence<Output = Callback, Error = SequenceBindingError>>;

#[derive(Debug, Default)]
struct KeyBindings {
    binds: HashSet<CallbackSequenceBinding>,
}

impl KeyBindings {
    fn set(&mut self, val: CallbackSequenceBinding) {
        self.binds.insert(val);
    }

    fn match_sequence(&self, sequence: &[Key]) -> (Vec<Callback>, bool) {
        let mut res = Vec::new();
        let mut partial_match_found = false;
        for bind in self.binds.iter() {
            match bind.match_sequence(sequence) {
                Ok(val, _) => {
                    res.push(val);
                    partial_match_found = true
                }
                Err(e) => match e {
                    SequenceBindingError::NotEnoghKeys => partial_match_found = true,
                    SequenceBindingError::MismatchedKey => {}
                },
            }
        }
        (res, partial_match_found)
    }
}

#[derive(Default)]
pub struct EditorBindings {
    pub normal: KeyBindings,
    pub insert: KeyBindings,
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
        let handler = match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
            Mode::Cell => todo!(),
        };
        let cb = loop {
            if sequence.is_empty() {
                break None;
            }
            let (cbs, partial_match) = handler.match_sequence(sequence);
            if !partial_match {
                sequence.remove(0);
                continue;
            }
        };
    }

    pub fn add_callback_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        cb: impl Into<Callback>,
    ) -> Result<(), BindingParseError> {
        let cb = cb.into();
        let sequence: Arc<[Key]> = parse_key_sequence(sequence)?.into();
        let modes = parse_modes(modes)?;

        for mode in modes {
            self.add_callback_binding(&mode, sequence.clone(), cb.clone());
        }

        Ok(())
    }

    pub fn add_callback_binding(
        &mut self,
        mode: &Mode,
        sequence: Arc<[Key]>,
        cb: impl Into<Callback>,
    ) {
        let cb = cb.into();
        let handle = SequenceBinding::from_sequence(sequence, cb);
        match mode {
            Mode::Normal => self.normal.set(Box::new(handle)),
            Mode::Insert => self.insert.set(Box::new(handle)),
            Mode::Cell => todo!(),
        }
    }
}
