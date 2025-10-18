pub mod bindings;
pub mod view;

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    editor::bindings::{BindingParseError, Callback, EditorBindings},
    key::Key,
    mode::Mode,
    table::{DataTable, cell::CellPos},
};

#[derive(Debug, Default)]
pub struct State {
    pub mode: Mode,
    pub table: DataTable<i64>,
    pub cursor: CellPos,
}

#[derive(Default)]
pub struct Editor {
    state: Arc<RwLock<State>>,
    bindings: EditorBindings,
    key_sequence: Vec<Key>,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_callback_binding(&mut self, mode: &Mode, sequence: &[Key], cb: Callback) {
        self.bindings.add_callback_binding(mode, sequence, cb);
    }

    pub fn add_callback_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        callback: Callback,
    ) -> Result<(), BindingParseError> {
        self.bindings
            .add_callback_bindings_str(modes, sequence, callback)
    }

    fn handle_sequence(&mut self) -> Option<Callback> {
        self.bindings
            .hadndle_sequence(&mut self.key_sequence, &self.state.read().unwrap().mode)
    }

    pub fn display_mode(&self) -> &str {
        match self.state.read().unwrap().mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Cell => "CELL",
            // _ => todo!(),
        }
    }

    pub fn display_sequence(&self) -> String {
        let mut s = String::new();
        for key in self.key_sequence.iter() {
            s += &format!("{key }");
        }
        s
    }
}

impl Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sequence: {:?}\n mode: {:?}",
            self.key_sequence,
            self.state.read().unwrap()
        )
    }
}
