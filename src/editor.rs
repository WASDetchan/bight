pub mod bindings;

use crate::{
    key::Key,
    mode::Mode,
    table::{DataTable, cell::CellPos},
};

#[derive(Debug, Default)]
pub struct EditorState {
    pub mode: Mode,
    pub table: DataTable<i64>,
    pub cursor: CellPos,
}

pub fn display_sequence(seq: &[Key]) -> String {
    let mut s = String::new();
    for key in seq.iter() {
        s += &format!("{key }");
    }
    s
}
