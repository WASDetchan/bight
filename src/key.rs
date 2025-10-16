use std::{collections::HashMap, fmt::Display};

use cursive::event::{self, Event};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Key(pub Event);

pub enum KeyString {
    Plain(String),
    Escape(String),
}

impl KeyString {
    pub fn into_inner(self) -> String {
        match self {
            Self::Plain(s) => s,
            Self::Escape(s) => s,
        }
    }

    pub fn inner_str(&self) -> &str {
        match self {
            Self::Plain(s) => &s,
            Self::Escape(s) => &s,
        }
    }
}

impl<T: Into<Event>> From<T> for Key {
    fn from(value: T) -> Self {
        let inner: Event = value.into();
        Key(inner)
    }
}

impl Key {
    fn format(&self) -> KeyString {
        use KeyString::{Escape, Plain};
        match self.0 {
            Event::CtrlChar(c) => Escape(format!("C-{}", Key::from(c).format().inner_str())),
            Event::AltChar(c) => Escape(format!("A-{}", Key::from(c).format().inner_str())),
            Event::Char(c) => match c {
                '<' => Escape("lt".into()),
                _ => Plain(format!("{c}")),
            },
            Event::Key(k) => match k {
                event::Key::Esc => Escape("Esc".into()),
                _ => todo!("All other special keys should be added"),
            },
            _ => todo!("Other events should be handled"),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format() {
            KeyString::Escape(s) => write!(f, "<{s}>"),
            KeyString::Plain(s) => write!(f, "{s}"),
        }
    }
}

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
