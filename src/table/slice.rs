pub mod iter;

use std::ops::RangeInclusive;

use iter::TableSliceIter;

use super::{
    Table,
    cell::{CellContent, CellPos},
};

pub type IdxRange = RangeInclusive<usize>;

pub struct SlicePos {
    pub start: CellPos,
    pub end: CellPos,
}

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
    pub fn new(pos: impl Into<SlicePos>, table: &'a T) -> Self {
        Self {
            pos: pos.into(),
            table,
        }
    }

    pub fn get(&self, pos: impl Into<CellPos>) -> Option<&'a CellContent<T::Item>> {
        self.table.get(pos.into())
    }
}

impl<'a, T: Table> IntoIterator for TableSlice<'a, T> {
    type IntoIter = TableSliceIter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
pub struct RowSlice<'a, T: Table> {
    inner: TableSlice<'a, T>,
}

impl<'a, T: Table> RowSlice<'a, T> {
    pub fn into_inner(self) -> TableSlice<'a, T> {
        self.inner
    }
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
