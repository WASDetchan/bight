use crate::table::Table;

use super::table::TableSlice;

/// A TableSlice that is guaranteed to be a single column (which means its start's and end's x
/// positions are the same)
///
/// Can be created from a TableSlice using TryFrom
pub struct ColSlice<'a, T: Table> {
    inner: TableSlice<'a, T>,
}

impl<'a, T: Table> ColSlice<'a, T> {
    pub fn into_inner(self) -> TableSlice<'a, T> {
        self.inner
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Given SlicePos is not a single column")]
pub struct ColSliceError;

impl<'a, T: Table> TryFrom<TableSlice<'a, T>> for ColSlice<'a, T> {
    type Error = ColSliceError;
    fn try_from(value: TableSlice<'a, T>) -> Result<Self, Self::Error> {
        if value.is_col() {
            Ok(Self { inner: value })
        } else {
            Err(ColSliceError)
        }
    }
}

impl<'a, T: Table> IntoIterator for ColSlice<'a, T> {
    type Item = <TableSlice<'a, T> as IntoIterator>::Item;
    type IntoIter = <TableSlice<'a, T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
