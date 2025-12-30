use std::fmt::Debug;
use std::str::FromStr;
use std::{collections::HashSet, fmt::Display};

const TABLE_CELL_PLACEHOLDER: &str = " ";

use super::{DataTable, Table};

use rkyv::{Archive, Deserialize, Serialize};
#[derive(Clone, Copy, Hash, PartialEq, Eq, Default, Archive, Serialize, Deserialize)]
#[rkyv(derive(PartialEq, Eq, Hash))]
pub struct CellPos {
    pub x: usize,
    pub y: usize,
}

impl Debug for CellPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<(usize, usize)> for CellPos {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CellPosParseError {
    #[error("CellPos str contained an invalid digit")]
    InvalidDidit,
}

const LETTER_BASE: u32 = 26;
impl FromStr for CellPos {
    type Err = CellPosParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letters = s
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase());

        let mut x = 0usize;
        for l in letters {
            x *= LETTER_BASE as usize;
            x += l
                .to_digit(LETTER_BASE + 10)
                .expect("Only letters can be in letters") as usize
                - 10;
        }

        let numbers = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .take_while(|c| c.is_ascii_digit());

        let mut y = 0usize;
        for n in numbers {
            y *= 10;
            y += n.to_digit(10).expect("Only digits can be in numbers") as usize;
        }

        let left = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .skip_while(|c| c.is_ascii_digit());

        if left.count() > 0 {
            Err(CellPosParseError::InvalidDidit)
        } else {
            Ok((x, y).into())
        }
    }
}

impl Display for CellPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = self.x;
        if x == 0 {
            write!(f, "A")?;
        }
        let mut chars = Vec::new();
        while x > 0 {
            let digit = x % LETTER_BASE as usize;
            let c = char::from_digit(digit as u32 + 10, LETTER_BASE + 10)
                .expect("digit is always less that LETTER_BASE")
                .to_ascii_uppercase();
            chars.push(c);
            x /= LETTER_BASE as usize;
        }
        for c in chars.into_iter().rev() {
            write!(f, "{c}")?;
        }
        write!(f, "{}", self.y)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Cell<I> {
    pub content: Option<I>,
    _dependencies: HashSet<CellPos>,
    _required_by: HashSet<CellPos>,
}

impl<I> Default for Cell<I> {
    fn default() -> Self {
        Self {
            content: None,
            _dependencies: HashSet::default(),
            _required_by: HashSet::default(),
        }
    }
}

pub enum CellContent<I> {
    Table(Box<dyn Table<Item = I> + Send + Sync + 'static>),
    Value(I),
}

impl<I: Debug> Debug for CellContent<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table(_) => write!(f, "{TABLE_CELL_PLACEHOLDER}"),
            Self::Value(v) => write!(f, "{v:?}"),
        }
    }
}
impl<I: Display> Display for CellContent<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table(_) => write!(f, "{TABLE_CELL_PLACEHOLDER}"),
            Self::Value(v) => write!(f, "{v}"),
        }
    }
}

impl<I: 'static + Send + Sync> CellContent<I> {
    pub fn empty_data_table() -> Self {
        Self::Table(Box::new(DataTable::<I>::new()))
    }
}

impl<I: Default> CellContent<I> {
    pub fn default_value() -> Self {
        Self::Value(I::default())
    }
}
