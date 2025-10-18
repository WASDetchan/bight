use std::fmt::Debug;

use super::table::TableSlice;
use crate::table::Table;

/// A TableSlice that is guaranteed to be a single row (which means its start's and end's y
/// positions are the same)
///
/// Can be created from a TableSlice using TryFrom
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
        if value.is_row() {
            Ok(Self { inner: value })
        } else {
            Err(RowSliceError)
        }
    }
}

impl<'a, T: Table> IntoIterator for RowSlice<'a, T> {
    type Item = <TableSlice<'a, T> as IntoIterator>::Item;
    type IntoIter = <TableSlice<'a, T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T: Table> Debug for RowSlice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RowSlice with {:?}", self.inner)
    }
}
