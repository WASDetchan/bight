pub mod vim_default;

use crate::{
    callback::OnKeyEventCallback as Callback,
    editor::mode::{Mode, ModeParseError, parse_modes},
    key::Key,
    key::sequence::{MatchSequence, SequenceBinding, SequenceMatchError, SequenceParseError},
};

#[derive(Default)]
pub struct KeyBindings {
    bindings: Vec<Box<dyn MatchSequence<Output = Callback>>>,
}

impl KeyBindings {
    pub fn push(&mut self, binding: Box<dyn MatchSequence<Output = Callback>>) {
        self.bindings.push(binding);
    }
    pub fn find(&self, sequence: &[Key]) -> Result<Callback, SequenceMatchError> {
        let mut found_hint = None;
        for binding in self.bindings.iter() {
            let res = binding.try_match(sequence);
            match res {
                Ok(cb) => return Ok(cb),
                Err(SequenceMatchError::CannotBeContined) => {}
                Err(SequenceMatchError::CanBeContined { hint }) => found_hint = Some(hint),
            }
        }
        let Some(hint) = found_hint else {
            return Err(SequenceMatchError::CannotBeContined);
        };
        Err(SequenceMatchError::CanBeContined { hint })
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
        let bindings = match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
            Mode::Cell => todo!(),
        };
        let cb = loop {
            let cb = bindings.find(sequence);
            if cb.is_ok() || sequence.is_empty() || cb.as_ref().is_err_and(|e| e.can_be_continued())
            {
                break cb;
            }
            sequence.remove(0);
        };
        match cb {
            Ok(cb) => {
                sequence.clear();
                Some(cb)
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
        let cb: Callback = cb.into();
        let binding = SequenceBinding::bind_str(sequence, cb)?;

        for mode in parse_modes(modes)? {
            self.add_sequence_handler(mode, Box::new(binding.clone()));
        }

        Ok(())
    }

    pub fn add_callback_binding(&mut self, mode: Mode, sequence: &[Key], cb: impl Into<Callback>) {
        let cb: Callback = cb.into();
        let binding = SequenceBinding::new(sequence.to_vec(), cb);
        self.add_sequence_handler(mode, Box::new(binding));
    }
    pub fn add_sequence_handler(
        &mut self,
        mode: Mode,
        binding: Box<dyn MatchSequence<Output = Callback>>,
    ) {
        match mode {
            Mode::Normal => self.normal.push(binding),
            Mode::Insert => self.insert.push(binding),
            Mode::Cell => todo!(),
        }
    }
}
