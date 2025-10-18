use crate::table::Table;

use super::table::TableSlice;

/// A TableSlice that is guaranteed to be a single row (which means it's start's and end's y
/// position are the same)
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
