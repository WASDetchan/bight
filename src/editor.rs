pub mod bindings;

use crate::{evaluator::EvaluatorTable, key::Key, mode::Mode, table::cell::CellPos};

// type CellType = String;

#[derive(Debug, Default)]
pub struct EditorState {
    pub expand: bool,
    pub mode: Mode,
    pub table: EvaluatorTable,
    pub cursor: CellPos,
}

pub fn display_sequence(seq: &[Key]) -> String {
    let mut s = String::new();
    for key in seq.iter() {
        s += &format!("{key }");
    }
    s
}
