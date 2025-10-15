use cursive::{Cursive, event::Event};

use crate::mode::HasMode;

pub trait AddModeCallback: HasMode {
    fn add_callback(
        &mut self,
        event: impl Into<Event>,
        cb: impl FnMut(&mut Cursive) + 'static + Sync + Send,
    );
    fn add_non_text_callback(
        &mut self,
        event: impl Into<Event>,
        mut cb: impl FnMut(&mut Cursive) + 'static + Sync + Send,
    ) {
        eprintln!("Mode callback added");
        let mode_cb = self.is_text_mode_closure();
        self.add_callback(event, move |s| {
            eprintln!("Mode quried");
            if !mode_cb() {
                cb(s);
            }
        });
    }
}
