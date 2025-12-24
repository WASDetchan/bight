use crate::table::{
    Table,
    slice::{IdxRange, col::ColSlice, row::RowSlice},
};

use super::TableSlice;

pub struct TableSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    current_row: Option<usize>,
    colums: IdxRange,
    rows: IdxRange,
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        let mut rows = value.row_indexes();
        Self {
            current_row: rows.next(),
            rows,
            colums: value.col_indexes(),
            slice: value,
        }
    }
}

impl<'a, T: Table> Iterator for TableSliceIter<'a, T> {
    type Item = Option<&'a T::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next_column = self.colums.next();
        while next_column.is_none() {
            self.current_row = self.rows.next();
            self.colums = self.slice.col_indexes();
            next_column = self.colums.next();
        }

        let next_column = next_column.expect("Loop couldn't have exited if next_column is None");

        Some(
            self.slice
                .get((next_column, self.current_row?))
                .expect("Only valid shift could have been requested"),
        )
    }
}
impl<'a, T: Table> IntoIterator for TableSlice<'a, T> {
    type IntoIter = TableSliceIter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
pub struct TableRowSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    rows: IdxRange,
}

impl<'a, T: Table> Iterator for TableRowSliceIter<'a, T> {
    type Item = RowSlice<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_row = self.rows.next()?;
        Some(
            TableSlice::new(
                (
                    (self.slice.pos.start.x, next_row),
                    (self.slice.pos.end.x, next_row + 1),
                ),
                self.slice.table,
            )
            .try_into()
            .expect("The created slice is guaranteed to be a single row"),
        )
    }
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableRowSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        let rows = value.row_indexes();
        Self { slice: value, rows }
    }
}

pub struct TableColSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    cols: IdxRange,
}

impl<'a, T: Table> Iterator for TableColSliceIter<'a, T> {
    type Item = ColSlice<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_col = self.cols.next()?;
        Some(
            TableSlice::new(
                (
                    (next_col, self.slice.pos.start.y),
                    (next_col + 1, self.slice.pos.end.y),
                ),
                self.slice.table,
            )
            .try_into()
            .expect("The created slice is guaranteed to be a single column"),
        )
    }
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableColSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        let cols = value.col_indexes();
        Self { slice: value, cols }
    }
}
