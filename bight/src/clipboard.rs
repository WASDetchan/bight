use std::{
    fmt::Debug,
    hash::{self, DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

pub trait ClipboardProvider {
    fn set_str(&mut self, v: &str);
    fn get_str(&mut self) -> Option<String>;
}

pub struct ArboardProvider {
    inner: Mutex<arboard::Clipboard>,
}

impl ClipboardProvider for ArboardProvider {
    fn set_str(&mut self, v: &str) {
        self.inner
            .lock()
            .unwrap()
            .set_text(v)
            .expect("Failed to set clipboard text");
    }
    fn get_str(&mut self) -> Option<String> {
        self.inner.lock().unwrap().get_text().ok()
    }
}

pub struct Clipboard {
    copied_val: Option<(Arc<str>, u64)>,
    inner: Box<dyn ClipboardProvider + Send + Sync>,
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cliboard, copied: {:?}", self.copied_val)
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            copied_val: None,
            inner: Box::new(ArboardProvider {
                inner: Mutex::new(
                    arboard::Clipboard::new().expect("Failed to initialize clipboard"),
                ),
            }),
        }
    }
}

impl Clipboard {
    pub fn with_provider(p: impl ClipboardProvider + Send + Sync + 'static) -> Self {
        Self {
            copied_val: None,
            inner: Box::new(p),
        }
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&mut self, v: Arc<str>) {
        let hash = {
            let mut hasher: hash::DefaultHasher = DefaultHasher::new();
            v.hash(&mut hasher);
            hasher.finish()
        };
        self.inner.set_str(&v);
        self.copied_val = Some((v, hash));
    }
    pub fn get(&mut self) -> Option<Arc<str>> {
        let cb_text = self.inner.get_str()?;
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
