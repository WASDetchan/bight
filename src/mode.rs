#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, thiserror::Error)]
pub enum ModeParseError {
    #[error("Passed str contained char '{0}' which cannot be mapped to a mode")]
    InvalidChar(char),
}

pub fn parse_modes(s: &str) -> Result<Vec<Mode>, ModeParseError> {
    let mut modes = Vec::new();
    for c in s.chars() {
        modes.push(match c {
            'n' => Mode::Normal,
            'i' => Mode::Insert,
            'c' => Mode::Cell,
            _ => return Err(ModeParseError::InvalidChar(c)),
        });
    }

    Ok(modes)
}
