//! Module with core Table traits and realted reexports
//!
//! Most operations in bight happen on tables - items that implment the [`Table`] trait. There are a
//! few datatypes related to the trait:
//! - [`CellPos`] is a type that represents some position in a table. It has 2 integer coordinates: x and
//!   y (alternatively column and row). It's used to get and set values of the tables.
//! - [`CellRange`] is a type that represents a rectangular area of a table. It's used to slice a
//!   table, producing a [`TableSlice`].
//! - [`TableRef`] represents a reference to some position in a table, allowing accessing the cells
//!   reative to it. [`TableRefMut`] is its mutable counterpart.
//! - [`TableSlice`] is a type that represents a reference to some rectangular area (range) of a
//!   table. It allows accessing the cells using coordintes relative to the start of the slice, and
//!   iterating over the cells in the area.
//! - [`DataTable`] is a simple Table implementor.

pub mod cell;
pub mod data_table;
pub mod reference;
pub mod slice;

use std::collections::HashMap;

pub use cell::CellPos;
pub use data_table::DataTable;
pub use reference::{TableRef, TableRefMut};
pub use slice::CellRange;
pub use slice::table::TableSlice;

// TODO: Wrap it in a struct and implement Table and TableMut
pub type HashTable<T> = HashMap<CellPos, T>;

/// Trait that represents a table. Table is similar to a HashMap with [`CellPos`] as its key. It
/// allows slicing it with a [`CellRange`] to get a reference to a bounded part of a table
pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&Self::Item>;
    fn ref_sh(&self, pos: CellPos) -> TableRef<'_, Self> {
        TableRef::new(self, pos)
    }
    fn slice(&self, range: impl Into<CellRange>) -> TableSlice<'_, Self> {
        let range = range.into();
        TableSlice::new(self.ref_sh(range.start), range.width, range.height)
    }
}

/// Mutable version of Table. Allows mutating the table's values. Mutable slices are not yet
/// supported
pub trait TableMut: Table {
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item>;
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>);
    fn ref_mut(&mut self, pos: CellPos) -> TableRefMut<'_, Self> {
        TableRefMut::new(self, pos)
    }
}
