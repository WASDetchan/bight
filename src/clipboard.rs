use std::{
    hash::{self, DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

pub struct Clipboard {
    copied_val: Option<(Arc<str>, u64)>,
    inner: arboard::Clipboard,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            copied_val: None,
            inner: arboard::Clipboard::new().expect("Failed to initialize clipboard"),
        }
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&mut self, v: Arc<str>) {
        let hash = {
            let mut hasher: hash::DefaultHasher = DefaultHasher::new();
            v.hash(&mut hasher);
            hasher.finish()
        };
        self.inner
            .set_text(v.to_string())
            .expect("Failed to set clipboard text");
        self.copied_val = Some((v, hash));
    }
    pub fn get(&mut self) -> Option<Arc<str>> {
        let cb_text = self.inner.get_text().ok()?;
        let cb_hash = {
            let mut hasher: hash::DefaultHasher = DefaultHasher::new();
            cb_text.hash(&mut hasher);
            hasher.finish()
        };
        if let Some((copied, hash)) = self.copied_val.as_ref()
            && *hash == cb_hash
            && copied.as_ref() == cb_text.as_str()
        {
            Some(copied.clone())
        } else {
            let text: Arc<str> = cb_text.into();
            self.copied_val = Some((text.clone(), cb_hash));
            Some(text)
        }
    }
}

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

pub fn get_clipboard() -> Option<Arc<str>> {
    let mut guard = CLIPBOARD.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Clipboard::new());
    }
    guard.as_mut().unwrap().get()
}

pub fn set_clipboard(v: Arc<str>) {
    let mut guard = CLIPBOARD.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Clipboard::new());
    }
    guard.as_mut().unwrap().set(v)
}
