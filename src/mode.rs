use std::sync::{Arc, RwLock};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Cell,
}

impl Mode {
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Insert)
    }
}
pub trait HasMode {
    fn get_mode(&self) -> Arc<RwLock<Mode>>;
    fn is_text_mode_closure(&self) -> impl Fn() -> bool + Send + Sync + 'static {
        let mode = self.get_mode();
        move || mode.read().unwrap().is_text()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModeParseError {
    #[error("Passed str contained char '{0}' which cannot be mapped to a mode")]
    InvalidChar(char),
}

pub fn parse_modes(s: &str) -> Result<Vec<Mode>, ModeParseError> {
    let mut modes = Vec::new();
    for c in s.chars() {
        modes.push( match c {
            'n' => Mode::Normal, 
            'i' => Mode::Insert,
            'c' => Mode::Cell,
            _ => return Err(ModeParseError::InvalidChar(c)),
        });
    }

    Ok(modes)
}
