use std::fmt::Display;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Key {
    event: KeyEvent,
}

impl Key {
    pub fn from_char(c: char) -> Self {
        Self {
            event: KeyEvent::from(KeyCode::Char(c)),
        }
    }
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
