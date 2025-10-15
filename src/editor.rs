pub struct KeyBindings {
    pub insert: Event,
    pub normal: Event,
    pub quit: Event,
}

use std::sync::{Arc, RwLock};

use cursive::{CursiveExt, event::Event};

use crate::{
    callback::AddModeCallback,
    mode::{HasMode, Mode},
};

#[derive(Default)]
pub struct Editor {
    inner: cursive::Cursive,
    mode: Arc<RwLock<Mode>>,
}

impl Editor {
    pub fn new(cursive: cursive::Cursive) -> Self {
        Self {
            inner: cursive,
            ..Default::default()
        }
    }
    pub fn run(&mut self) {
        self.inner.run();
    }

    pub fn add_bindings(&mut self, bindings: KeyBindings) {
        let mode = Arc::clone(&self.mode);
        self.add_non_text_callback(bindings.insert, move |_| {
            *mode.write().unwrap() = Mode::Insert;
        });
        let mode = Arc::clone(&self.mode);
        self.add_non_text_callback(bindings.normal,  move|_| {
            *mode.write().unwrap() = Mode::Normal;
        });
        self.add_callback(bindings.quit, |s| s.quit());
    }

    fn set_mode(&self, mode: Mode) {
        *self.mode.write().unwrap() = mode;
    }
}

impl HasMode for Editor {
    fn get_mode(&self) -> Arc<RwLock<Mode>> {
        Arc::clone(&self.mode)
    }
}

impl AddModeCallback for Editor {
    fn add_callback(
        &mut self,
        event: impl Into<Event>,
        cb: impl FnMut(&mut cursive::Cursive) + 'static + Sync + Send,
    ) {
        self.inner.add_global_callback(event, cb);
    }
}
