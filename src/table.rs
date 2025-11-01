pub mod cell;
pub mod slice;

use cell::{Cell, CellContent, CellPos};

pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&CellContent<Self::Item>>;
    fn set(&mut self, pos: CellPos, item: Option<CellContent<Self::Item>>);
}

#[derive(Debug)]
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
