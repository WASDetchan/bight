use std::rc::Rc;

use crate::{app::AppState, editor::EditorState, key::KeyTree};

pub struct StateCallback<S>(pub Rc<dyn Fn(&mut S)>);

impl<S> Clone for StateCallback<S> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<S> StateCallback<S> {
    pub fn new(f: impl Fn(&mut S) + 'static) -> Self {
        Self(Rc::new(f))
    }
}

pub type EditorStateCallback = StateCallback<EditorState>;
pub type AppStateCallback = StateCallback<AppState>;

#[derive(Clone)]
pub enum OnKeyEventCallback {
    EditorStateChanage(EditorStateCallback),
    AppStateChange(AppStateCallback),
}

impl From<EditorStateCallback> for OnKeyEventCallback {
    fn from(value: EditorStateCallback) -> Self {
        Self::EditorStateChanage(value)
    }
}

impl From<AppStateCallback> for OnKeyEventCallback {
    fn from(value: AppStateCallback) -> Self {
        Self::AppStateChange(value)
    }
}

pub type EventHandlers = Vec<OnKeyEventCallback>;

pub type KeyBindTree = KeyTree<EventHandlers>;
