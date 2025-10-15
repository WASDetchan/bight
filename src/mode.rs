use std::sync::{Arc, RwLock};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Cell,
}

impl Mode {
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Insert)
    }
}
pub trait HasMode {
    fn get_mode(&self) -> Arc<RwLock<Mode>>;
    fn is_text_mode_closure(&self) -> impl Fn() -> bool + Send + Sync + 'static {
        let mode = self.get_mode();
        move || mode.read().unwrap().is_text()
    }
}
