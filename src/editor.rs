pub mod bindings;
pub mod view;

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use cursive::Cursive;

use crate::{
    editor::bindings::{BindingParseError, Callback, EditorBindings},
    key::Key,
    mode::Mode,
};

#[derive(Debug, Clone)]
pub enum EditorCommand {
    NormalMode,
    InsertMode,
    Quit,
}

impl EditorCommand {
    pub fn make_callback(self, editor: &Editor) -> Callback {
        editor.make_editor_command_callback(self)
    }
}

#[derive(Default)]
pub struct Editor {
    mode: Arc<RwLock<Mode>>,
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

    pub fn add_command_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        command: EditorCommand,
    ) -> Result<(), BindingParseError> {
        let callback = self.make_editor_command_callback(command);
        self.bindings
            .add_callback_bindings_str(modes, sequence, callback)
    }

    pub fn add_command_binding(&mut self, mode: &Mode, sequence: &[Key], command: EditorCommand) {
        let callback = self.make_editor_command_callback(command);
        self.bindings.add_callback_binding(mode, sequence, callback);
    }

    pub fn make_editor_command_callback(&self, command: EditorCommand) -> Callback {
        match command {
            EditorCommand::NormalMode => {
                self.make_callback(|mode, _| *mode.write().unwrap() = Mode::Normal)
            }
            EditorCommand::InsertMode => {
                self.make_callback(|mode, _| *mode.write().unwrap() = Mode::Insert)
            }
            EditorCommand::Quit => self.make_callback(|_, s| s.quit()),
        }
    }

    pub fn make_callback(
        &self,
        cb: impl Fn(&Arc<RwLock<Mode>>, &mut Cursive) + Send + Sync + 'static,
    ) -> Callback {
        let mode = Arc::clone(&self.mode);
        Arc::new(move |s| cb(&mode, s))
    }

    pub fn set_mode(&self, mode: Mode) {
        *self.mode.write().unwrap() = mode;
    }

    fn handle_sequence(&mut self) -> Option<Callback> {
        self.bindings
            .hadndle_sequence(&mut self.key_sequence, &self.mode.read().unwrap())
    }
}

impl Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sequence: {:?}\n mode: {:?}",
            self.key_sequence,
            *self.mode.read().unwrap()
        )
    }
}
