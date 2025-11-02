use std::{collections::HashMap, fmt::Display};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Key {
    event: KeyEvent,
}

impl Key {
    fn format(&self) -> KeyString {
        use KeyString::{Escape, Plain};

        let mods = self.event.modifiers;
        let code = self.event.code;

        let mut plain = true;
        let mut s = String::new();

        mods.iter_names().for_each(|x| {
            match x.1 {
                KeyModifiers::SHIFT => match code {
                    KeyCode::Char(c) if c != '<' => {
                        return;
                    }
                    _ => s += "S-",
                },
                KeyModifiers::CONTROL => s += "C-",
                KeyModifiers::ALT => s += "A-",
                KeyModifiers::SUPER => todo!(),
                KeyModifiers::HYPER => todo!(),
                KeyModifiers::META => s += "M-",
                _ => unreachable!(),
            };
            plain = false;
        });

        match code {
            KeyCode::Char(c) => match c {
                '<' => {
                    s += "lt";
                    plain = false;
                }
                _ => s += &String::from(c),
            },
            KeyCode::Esc => s += "Esc",
            _ => todo!("handle other keycodes"),
        }

        if plain { Plain(s) } else { Escape(s) }
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
impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        Self { event: value }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Given event is not a key event")]
pub struct EventToKeyConversionError;

impl TryFrom<Event> for Key {
    type Error = EventToKeyConversionError;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        if let Event::Key(ke) = value {
            Ok(ke.into())
        } else {
            Err(EventToKeyConversionError)
        }
    }
}
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
            Self::Plain(s) => s,
            Self::Escape(s) => s,
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
        result.push(Key {
            event: KeyEvent::from(KeyCode::Char(c)),
        });
    }

    Ok(result)
}
