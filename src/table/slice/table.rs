pub mod iter;

use crate::table::{
    Table,
    cell::{CellContent, CellPos},
};

use super::{IdxRange, SlicePos};

/// A slice of a table (a wide pointer to a table that has a starting cell and an ending cell)
///
/// The slice is inclusive (the end cell, its row and col are included)
/// Both end's coordinates are greater or equal to the corresponding start's coordinates (end must
/// be to the down-right of the start)
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

    pub fn get(&self, pos: impl Into<CellPos>) -> Option<Option<&'a CellContent<T::Item>>> {
        let pos: CellPos = pos.into();
        Some(self.table.get(self.pos.shift_to_pos(pos)?))
    }
    pub fn is_col(&self) -> bool {
        self.pos.start.x == self.pos.end.x
    }

    pub fn is_row(&self) -> bool {
        self.pos.start.y == self.pos.end.y
    }

    pub fn row_indexes(&self) -> IdxRange {
        self.pos.rows()
    }

    pub fn col_indexes(&self) -> IdxRange {
        self.pos.columns()
    }
}
