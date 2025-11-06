use std::fmt::Debug;
use std::str::FromStr;
use std::{collections::HashSet, fmt::Display};

const TABLE_CELL_PLACEHOLDER: &str = " ";
use super::{DataTable, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct CellPos {
    pub x: usize,
    pub y: usize,
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

impl FromStr for CellPos {
    type Err = CellPosParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        eprintln!("parsing str: {s}");
        let letters = s
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase());

        const LETTER_BASIS: u32 = 26;
        let mut x = 0usize;
        for l in letters {
            x *= LETTER_BASIS as usize;
            x += l
                .to_digit(LETTER_BASIS + 10)
                .expect("Only letters can be in letters") as usize
                - 10;
        }

        let numbers = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .take_while(|c| c.is_digit(10));

        let mut y = 0usize;
        for n in numbers {
            y *= 10;
            y += n.to_digit(10).expect("Only digits can be in numbers") as usize;
        }

        let left = s
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .skip_while(|c| c.is_digit(10));

        eprintln!("got x: {x} y: {y}");
        if left.count() > 0 {
            Err(CellPosParseError::InvalidDidit)
        } else {
            Ok((x, y).into())
        }
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
