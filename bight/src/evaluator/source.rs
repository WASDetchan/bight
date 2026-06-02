use std::collections::hash_map;

use crate::{
    sync::RcStr,
    table::{CellPos, Table, TableMut},
};

struct EditStep {
    cell: CellPos,
    from: Option<RcStr>,
    to: Option<RcStr>,
}

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Default, Clone, PartialEq, Eq,
)]
pub struct SourceTable {
    inner: crate::table::HashTable<RcStr>,
}

impl SourceTable {
    pub fn inner_iter(&self) -> hash_map::Iter<'_, CellPos, RcStr> {
        self.inner.iter()
    }
    pub fn into_inner_iter(self) -> hash_map::IntoIter<CellPos, RcStr> {
        self.inner.into_iter()
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_source(source: crate::table::HashTable<RcStr>) -> Self {
        Self { inner: source }
    }
}

impl Table for SourceTable {
    type Item = RcStr;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        self.inner.get(&pos)
    }
}

impl TableMut for SourceTable {
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item> {
        self.inner.get_mut(&pos)
    }
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>) {
        match item {
            None => self.inner.remove(&pos),
            Some(item) => self.inner.insert(pos, item),
        };
    }
}
