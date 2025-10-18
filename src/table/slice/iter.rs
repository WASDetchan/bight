use super::{
    super::cell::CellContent,
    Table, {IdxRange, RowSlice, TableSlice},
};

pub struct TableSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    current_row: usize,
    colums: IdxRange,
    rows: IdxRange,
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        Self {
            rows: value.pos.rows(),
            colums: value.pos.columns(),
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
            self.colums = self.slice.pos.columns();
            next_column = self.colums.next();
        }

        let next_column = next_column.expect("Loop couldn't be exited if next_column is None");

        Some(self.slice.get((next_column, self.current_row)))
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
