pub mod bindings;
pub mod mode;

use crate::{
    clipboard::{Clipboard, ClipboardProvider},
    evaluator::EvaluatorTable,
    key::Key,
    table::cell::CellPos,
};
use mode::Mode;

#[derive(Debug, Default)]
pub struct EditorState {
    pub expand: bool,
    pub mode: Mode,
    pub table: EvaluatorTable,
    pub cursor: CellPos,
    pub clipboard: Clipboard,
}

impl EditorState {
    pub fn with_clipboard(p: impl ClipboardProvider + Send + Sync + 'static) -> Self {
        Self {
            clipboard: Clipboard::with_provider(p),
            ..Default::default()
        }
    }
}

pub fn display_sequence(seq: &[Key]) -> String {
    let mut s = String::new();
    for key in seq.iter() {
        s += &format!("{key }");
    }
    s
}
