pub mod view;

use std::{collections::HashSet, ops::RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Table(Box<dyn Table<Item = I>>),
    Value(I),
}

impl<I: 'static> CellContent<I> {
    pub fn empty_data_table() -> Self {
        Self::Table(Box::new(DataTable::<I>::new()))
    }
}

impl<I: Default> CellContent<I> {
    pub fn default_value() -> Self {
        Self::Value(I::default())
    }
}

pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&CellContent<Self::Item>>;
    fn set(&mut self, pos: CellPos, item: Option<CellContent<Self::Item>>);
}

pub struct DataTable<I> {
    data: Vec<Vec<Cell<I>>>,
}

impl<I> DataTable<I> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I> Default for DataTable<I> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<I> Table for DataTable<I> {
    type Item = I;
    fn get(&self, pos: CellPos) -> Option<&CellContent<Self::Item>> {
        self.data.get(pos.x)?.get(pos.y)?.content.as_ref()
    }
    fn set(&mut self, pos: CellPos, item: Option<CellContent<Self::Item>>) {
        if self.data.len() <= pos.x {
            self.data.resize_with(pos.x + 1, Vec::default);
        }
        if self.data[pos.x].len() <= pos.y {
            self.data[pos.x].resize_with(pos.y + 1, Cell::default);
        }

        self.data[pos.x][pos.y].content = item;
    }
}

pub struct SlicePos {
    pub start: CellPos,
    pub end: CellPos,
}

pub type IdxRange = RangeInclusive<usize>;

impl SlicePos {
    pub fn new<A: Into<CellPos>, B: Into<CellPos>>(start: A, end: B) -> Self {
        let mut start: CellPos = start.into();
        let mut end: CellPos = end.into();

        if start.x > end.x {
            std::mem::swap(&mut start.x, &mut end.x);
        }
        if start.y > end.y {
            std::mem::swap(&mut start.y, &mut end.y);
        }

        Self { start, end }
    }
    pub fn is_inside<P: Into<CellPos>>(&self, pos: P) -> bool {
        let p: CellPos = pos.into();
        (p.x >= self.start.x) && (p.y >= self.start.y) && (p.x <= self.end.x) && (p.y <= self.end.y)
    }

    pub fn columns(&self) -> IdxRange {
        (self.start.x)..=(self.end.x)
    }

    pub fn rows(&self) -> IdxRange {
        (self.start.y)..=(self.end.y)
    }
}

impl<A: Into<CellPos>, B: Into<CellPos>> From<(A, B)> for SlicePos {
    fn from(value: (A, B)) -> Self {
        Self::new(value.0, value.1)
    }
}

pub struct TableSlice<'a, T: Table> {
    pos: SlicePos,
    table: &'a T,
}

impl<'a, T: Table> TableSlice<'a, T> {
    pub fn new<P: Into<SlicePos>>(pos: P, table: &'a T) -> Self {
        Self {
            pos: pos.into(),
            table,
        }
    }
}

pub struct RowSlice<'a, T: Table> {
    inner: TableSlice<'a, T>,
}

#[derive(Debug, thiserror::Error)]
#[error("Given SlicePos is not a single row")]
pub struct RowSliceError;

impl<'a, T: Table> TryFrom<TableSlice<'a, T>> for RowSlice<'a, T> {
    type Error = RowSliceError;
    fn try_from(value: TableSlice<'a, T>) -> Result<Self, Self::Error> {
        if value.pos.start.y != value.pos.end.y {
            Err(RowSliceError)
        } else {
            Ok(Self { inner: value })
        }
    }
}

pub struct RowIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    rows: IdxRange,
}

impl<'a, T: Table> Iterator for RowIter<'a, T> {
    type Item = RowSlice<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_row = self.rows.next()?;
        Some(
            TableSlice::new(
                (
                    (self.slice.pos.start.x, next_row),
                    (self.slice.pos.end.x, next_row),
                ),
                self.slice.table,
            )
            .try_into()
            .expect("The created slice is guaranteed to be a single row"),
        )
    }
}
