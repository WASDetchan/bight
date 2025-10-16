use std::sync::{Arc, RwLock};

use cursive::{
    Cursive, View,
    event::{Event, EventResult},
};

use crate::{
    key::{Key, KeySequenceError, KeyTree, SequenceParseError, parse_key_sequence},
    mode::{Mode, ModeParseError, parse_modes},
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

type Callback = Arc<dyn Fn(&mut Cursive) + Send + Sync + 'static>;
type KeyBindTree = KeyTree<Callback>;

#[derive(Default)]
pub struct EditorBindings {
    pub normal: KeyBindTree,
    pub insert: KeyBindTree,
}

#[derive(Default)]
pub struct Editor {
    mode: Arc<RwLock<Mode>>,
    bindings: EditorBindings,
    key_sequence: Vec<Key>,
}

#[derive(Debug, thiserror::Error)]
pub enum BindingParseError {
    #[error(transparent)]
    KeySequenceParseError(#[from] SequenceParseError),
    #[error(transparent)]
    ModeParseError(#[from] ModeParseError),
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_callback_binding(&mut self, mode: Mode, sequence: &[Key], cb: Callback) {
        match mode {
            Mode::Normal => self.bindings.normal.map(sequence, Some(cb)),
            Mode::Insert => self.bindings.insert.map(sequence, Some(cb)),
            Mode::Cell => todo!(),
        }
    }

    pub fn add_command_bindings_str(
        &mut self,
        modes: &str,
        sequence: &str,
        command: EditorCommand,
    ) -> Result<(), BindingParseError> {
        let sequence = parse_key_sequence(sequence)?;
        let modes = parse_modes(modes)?;

        for mode in modes {
            self.add_command_binding(mode, &sequence, command.clone());
        }

        Ok(())
    }
    pub fn add_command_binding(&mut self, mode: Mode, sequence: &[Key], command: EditorCommand) {
        match mode {
            Mode::Normal => self
                .bindings
                .normal
                .map(sequence, Some(command.make_callback(self))),
            Mode::Insert => self
                .bindings
                .insert
                .map(sequence, Some(command.make_callback(self))),
            Mode::Cell => todo!(),
        }
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

    fn get_current_key_tree<'a>(
        mode: &Arc<RwLock<Mode>>,
        bindings: &'a EditorBindings,
    ) -> &'a KeyBindTree {
        match *mode.read().unwrap() {
            Mode::Normal => &bindings.normal,
            Mode::Insert => &bindings.insert,
            Mode::Cell => todo!(),
        }
    }

    fn check_sequence(&mut self) -> Option<Callback> {
        let tree = Self::get_current_key_tree(&self.mode, &self.bindings);
        let cb = loop {
            let cb = tree.find(&self.key_sequence);
            if cb.is_ok()
                || cb.is_err_and(|e| e != KeySequenceError::InvalidSequence)
                || self.key_sequence.is_empty()
            {
                break cb;
            }
            self.key_sequence.remove(0);
        };
        match cb {
            Ok(cb) => {
                self.key_sequence.clear();

                Some(Arc::clone(cb))
            }
            Err(_) => None,
        }
    }

    pub fn dbg(&self) -> String {
        format!(
            "sequence: {:?}\n mode: {:?}",
            self.key_sequence,
            *self.mode.read().unwrap()
        )
    }
}

pub struct EditorView {
    editor: Arc<RwLock<Editor>>,
}

impl EditorView {
    pub fn new(editor: Editor) -> Self {
        Self {
            editor: Arc::new(RwLock::new(editor)),
        }
    }
}

impl View for EditorView {
    fn draw(&self, printer: &cursive::Printer) {
        let seq = self.editor.read().unwrap().dbg();
        printer.print((0, 0), &seq);
    }

    fn on_event(&mut self, event: Event) -> cursive::event::EventResult {
        let mut editor = self.editor.write().unwrap();
        editor.key_sequence.push(Key(event));
        if let Some(cb) = editor.check_sequence() {
            EventResult::with_cb(move |s| (cb)(s))
        } else {
            EventResult::consumed()
        }
    }
}
