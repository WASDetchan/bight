pub mod bindings;
pub mod mode;

use crate::{evaluator::EvaluatorTable, key::Key, table::cell::CellPos};
use mode::Mode;

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
