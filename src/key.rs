use std::collections::HashMap;

use cursive::event::Event;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Key(pub Event);

pub enum KeyTree<T> {
    Value(T),
    Subtree(HashMap<Key, KeyTree<T>>),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone, Copy)]
pub enum KeySequenceError {
    #[error("The given key sequence did not lead to a value")]
    InvalidSequence,
    #[error("The given key sequence may be continued to lead to a value")]
    MayBeContinued,
}

impl<T> Default for KeyTree<T> {
    fn default() -> Self {
        Self::Subtree(HashMap::default())
    }
}

impl<T> KeyTree<T> {
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }
    pub fn find(&self, sequence: &[Key]) -> Result<&T, KeySequenceError> {
        use KeySequenceError::{InvalidSequence, MayBeContinued};
        use KeyTree::{Subtree, Value};

        match self {
            Value(val) => {
                if sequence.is_empty() {
                    Ok(val)
                } else {
                    Err(InvalidSequence)
                }
            }
            Subtree(tree) => {
                if sequence.is_empty() {
                    Err(MayBeContinued)
                } else {
                    let Some(next) = tree.get(&sequence[0]) else {
                        return Err(InvalidSequence);
                    };
                    next.find(&sequence[1..])
                }
            }
        }
    }

    pub fn map(&mut self, sequence: &[Key], value: Option<T>) {
        match value {
            None => _ = self.remove_value(sequence),
            Some(value) => self.set_value(sequence, value),
        }
    }

    pub fn set_value(&mut self, sequence: &[Key], value: T) {
        if sequence.is_empty() {
            let _ = std::mem::replace(self, Self::Value(value));
        } else {
            if self.is_value() {
                let _ = std::mem::take(self);
            }
            match self {
                Self::Value(_) => unreachable!(), // Value is changed to default (Subtree) above
                Self::Subtree(tree) => {
                    let next = tree
                        .entry(sequence[0].clone())
                        .or_insert(KeyTree::default());
                    next.set_value(&sequence[1..], value);
                }
            }
        }
    }

    pub fn remove_value(&mut self, sequence: &[Key]) -> Option<KeyTree<T>> {
        if sequence.is_empty() {
            panic!("Tried removing an empty sequence!");
        }
        match self {
            Self::Value(_) => None,
            Self::Subtree(tree) => {
                if sequence.len() == 1 {
                    tree.remove(&sequence[0])
                } else {
                    let next = tree.get_mut(&sequence[0])?;
                    next.remove_value(&sequence[1..])
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SequenceParseError {}

pub fn parse_key_sequence(sequence: &str) -> Result<Vec<Key>, SequenceParseError> {
    let mut result = Vec::new();

    for c in sequence.chars() {
        result.push(Key(c.into()));
    }

    Ok(result)
}
