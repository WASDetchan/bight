pub mod cell;
pub mod slice;

use cell::{Cell, CellPos};
use slice::table::TableSlice;

pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&Self::Item>;
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item>;
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>);
}

#[derive(Debug)]
pub struct DataTable<I> {
    data: Vec<Vec<Cell<I>>>,
}

impl<I> DataTable<I> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a  table slice that is guaranteed to contain every set element of this table (but
    /// doesn't guarantee that every element of slice is set)
    /// Return None if the table is empty
    pub fn full_slice(&self) -> Option<TableSlice<Self>> {
        let rows = self.data.len();
        let cols = self.data.iter().map(|v| v.len()).max().unwrap_or(0);
        (rows > 0 && cols > 0).then(|| TableSlice::new(((0, 0), (cols - 1, rows - 1)), self))
    }
}

impl<I> Default for DataTable<I> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<I> Table for DataTable<I> {
    type Item = I;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        self.data.get(pos.x)?.get(pos.y)?.content.as_ref()
    }
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item> {
        self.data.get_mut(pos.x)?.get_mut(pos.y)?.content.as_mut()
    }
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>) {
        if self.data.len() <= pos.x {
            self.data.resize_with(pos.x + 1, Vec::default);
        }
        if self.data[pos.x].len() <= pos.y {
            self.data[pos.x].resize_with(pos.y + 1, Cell::default);
        }

        self.data[pos.x][pos.y].content = item;
    }
}
