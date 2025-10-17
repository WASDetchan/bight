use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPos {
    pub x: usize,
    pub y: usize,
}

pub struct Cell<I> {
    pub content: Option<CellContent<I>>,
    _dependencies: HashSet<CellPos>,
    _required_by: HashSet<CellPos>,
}

impl<I> Default for Cell<I> {
    fn default() -> Self {
        Self {
            content: None,
            _dependencies: HashSet::default(),
            _required_by: HashSet::default(),
        }
    }
}

pub enum CellContent<I> {
    Table(Box<dyn Table<Item = I>>),
    Value(I),
}

impl<I: 'static> CellContent<I> {
    pub fn empty_data_table() -> Self {
        Self::Table(Box::new(DataTable::<I>::new()))
    }
}

impl<I: Default> CellContent<I> {
    pub fn default_value() -> Self {
        Self::Value(I::default())
    }
}

pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&CellContent<Self::Item>>;
    fn set(&mut self, pos: CellPos, item: Option<CellContent<Self::Item>>);
}

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
