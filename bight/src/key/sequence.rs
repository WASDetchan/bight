use crate::key::Key;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SequenceMatchError {
    #[error("The given key sequence did not match, but can match if continued")]
    CanBeContined { hint: String },
    #[error("The given key sequence cannot match")]
    CannotBeContined,
}

impl SequenceMatchError {
    pub fn can_be_continued(&self) -> bool {
        !self.eq(&Self::CannotBeContined)
    }
}

pub trait MatchSequence {
    type Output;
    fn try_match(&self, sequence: &[Key]) -> Result<Self::Output, SequenceMatchError>;
}

#[derive(Debug, Clone)]
pub struct SequenceBinding<T> {
    item: T,
    sequence: Vec<Key>,
}

impl<T> SequenceBinding<T> {
    pub fn new(sequence: Vec<Key>, item: T) -> Self {
        Self { item, sequence }
    }
    pub fn bind_str(sequence_str: &str, item: T) -> Result<Self, SequenceParseError> {
        let sequence = parse_key_sequence(sequence_str)?;
        Ok(Self::new(sequence, item))
    }
}

impl<T: Clone> MatchSequence for SequenceBinding<T> {
    type Output = T;
    fn try_match(&self, sequence: &[Key]) -> Result<Self::Output, SequenceMatchError> {
        for (idx, expected_key) in self.sequence.iter().enumerate() {
            let key = sequence.get(idx).ok_or(SequenceMatchError::CanBeContined {
                hint: format_sequence(&self.sequence[idx..]),
            })?;
            if key != expected_key {
                return Err(SequenceMatchError::CannotBeContined);
            }
        }
        Ok(self.item.clone())
    }
}

pub fn format_sequence(sequence: &[Key]) -> String {
    sequence
        .iter()
        .fold(String::new(), |s, k| format!("{}{}", s, k))
}

#[derive(Debug, thiserror::Error)]
pub enum SequenceParseError {}

pub fn parse_key_sequence(sequence: &str) -> Result<Vec<Key>, SequenceParseError> {
    let mut result = Vec::new();

    for c in sequence.chars() {
        result.push(Key::from_char(c));
    }

    Ok(result)
}
