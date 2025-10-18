use std::collections::HashSet;
use std::fmt::Debug;

use super::view::TABLE_CELL_PLACEHOLDER;
use super::{DataTable, Table};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

#[derive(Debug)]
pub struct Cell<I> {
    pub content: Option<CellContent<I>>,
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
