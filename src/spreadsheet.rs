use cursive::reexports::ahash::HashSet;


pub struct Spreadsheet<C> {
    pub data: Vec<Vec<Cell<C>>>,
}

impl<C> Spreadsheet<C> {
    pub fn get(&self, pos: CellPos) -> Option<&C> {
        self.data.get(pos.x)?.get(pos.y)?.content.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPos{
    pub x: usize,
    pub y: usize,
}
pub struct Cell<C> {
    pub content: Option<C>,
    _dependencies: HashSet<CellPos>,
    _required_by: HashSet<CellPos>,
}

