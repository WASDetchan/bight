use crate::table::{
    Table,
    cell::CellContent,
    slice::{IdxRange, row::RowSlice},
};

use super::TableSlice;

pub struct TableSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    current_row: usize,
    colums: IdxRange,
    rows: IdxRange,
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        Self {
            rows: value.row_indexes(),
            colums: value.col_indexes(),
            slice: value,
            current_row: 0,
        }
    }
}

impl<'a, T: Table> Iterator for TableSliceIter<'a, T> {
    type Item = Option<&'a CellContent<T::Item>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next_column = self.colums.next();
        while next_column.is_none() {
            self.current_row = self.rows.next()?;
            self.colums = self.slice.col_indexes();
            next_column = self.colums.next();
        }

        let next_column = next_column.expect("Loop couldn't be exited if next_column is None");

        Some(
            self.slice
                .get((next_column, self.current_row))
                .expect("Only valid shift could have been requested"),
        )
    }
}

pub struct ByRowTableSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    rows: IdxRange,
}

impl<'a, T: Table> Iterator for ByRowTableSliceIter<'a, T> {
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
impl<'a, T: Table> IntoIterator for TableSlice<'a, T> {
    type IntoIter = TableSliceIter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
