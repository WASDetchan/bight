use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};

use super::Key;

#[derive(Debug, thiserror::Error)]
pub enum SequenceParseError {}

pub fn parse_key_sequence(sequence: &str) -> Result<Vec<Key>, SequenceParseError> {
    let mut result = Vec::new();

    for c in sequence.chars() {
        result.push(Key {
            event: KeyEvent::from(KeyCode::Char(c)),
        });
    }

    Ok(result)
}

pub trait MatchKeySequence {
    type Error;
    type Output;
    fn match_sequence(&self, sequence: &[Key]) -> Result<Self::Output, Self::Error>;
    fn sequence_leftover<'a>(&self, sequence: &'a [Key]) -> Result<&'a [Key], Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum SequenceBindingError {
    #[error("Given sequence did not match the binding, but could match if continued")]
    NotEnoghKeys,
    #[error("Given sequence did not match the binding, and cannot match if continued")]
    MismatchedKey,
}

pub struct SequenceBinding<T> {
    output: T,
    sequence: Arc<[Key]>,
}

impl<T: Clone> MatchKeySequence for SequenceBinding<T> {
    type Error = SequenceBindingError;
    type Output = T;
    fn match_sequence<'a>(
        &self,
        sequence: &'a [Key],
    ) -> Result<(Self::Output, &'a [Key]), Self::Error> {
        let mut sequence = sequence;
        for key in self.sequence.iter() {
            let Some(k) = sequence.first() else {
                return Err(SequenceBindingError::NotEnoghKeys);
            };
            if k != key {
                return Err(SequenceBindingError::MismatchedKey);
            }
            sequence = &sequence[1..];
        }
        Ok(self.output.clone())
    }
    fn sequence_leftover<'a>(&self, sequence: &'a [Key]) -> Result<&'a [Key], Self::Error> {
        let mut sequence = sequence;
        for key in self.sequence.iter() {
            if sequence.first().is_none_or(|k| k != key) {
                return Err(SequenceBindingError);
            }
            sequence = &sequence[1..];
        }
        Ok(sequence)
    }
}

impl<T> SequenceBinding<T> {
    pub fn from_str(s: &str, value: T) -> Result<Self, SequenceParseError> {
        let sequence = parse_key_sequence(s)?.into();
        Ok(Self {
            sequence,
            output: value,
        })
    }
    pub fn from_sequence(s: impl Into<Arc<[Key]>>, value: T) -> Self {
        Self {
            sequence: s.into(),
            output: value,
        }
    }
}
