use std::{
    fmt::Debug,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError},
};

use cursive::{View, view::ViewWrapper, views::LinearLayout};

#[derive(Default, Debug)]
pub struct ChildView<I> {
    inner: RwLock<I>,
}

impl<I> From<I> for ChildView<I> {
    fn from(value: I) -> Self {
        Self {
            inner: RwLock::new(value),
        }
    }
}

impl<I> ChildView<I> {
    fn try_read(&self) -> Option<RwLockReadGuard<'_, I>> {
        match self.inner.try_read() {
            Err(e) => match &e {
                TryLockError::WouldBlock => None,
                TryLockError::Poisoned(_) => panic!("Child View Lock poisoned: {e}"),
            },
            Ok(v) => Some(v),
        }
    }

    fn try_write(&self) -> Option<RwLockWriteGuard<'_, I>> {
        match self.inner.try_write() {
            Err(e) => match &e {
                TryLockError::Poisoned(_) => panic!("Child View Lock poisoned: {e}"),
                TryLockError::WouldBlock => None,
            },
            Ok(v) => Some(v),
        }
    }
}

#[derive(Debug)]
pub struct Child<I>(Arc<ChildView<I>>);
impl<I> Clone for Child<I> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<I> Child<I> {
    pub fn new(view: ChildView<I>) -> Self {
        Self(Arc::new(view))
    }
}

impl<I: Default> Default for Child<I> {
    fn default() -> Self {
        Self::new(I::default().into())
    }
}

impl<I: View> ViewWrapper for Child<I> {
    type V = I;
    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        self.0.try_read().map(|v| f(&v))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        self.0.try_write().map(|mut v| f(&mut v))
    }
}

type Grid<T> = Vec<Vec<T>>;

/// Layout that places its children on a grid
pub struct GridLayout<I: View> {
    children: Grid<Child<I>>,
    view: LinearLayout,
}

impl<I: View + Debug> Debug for GridLayout<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GridLayout: Children: {:?}", self.children)
    }
}

impl<I: View> GridLayout<I> {
    fn make_view(children: &[Vec<Child<I>>]) -> LinearLayout {
        let mut layout = LinearLayout::vertical();
        children.iter().for_each(|row| {
            let mut row_layout = LinearLayout::horizontal();
            row.iter().for_each(|c| {
                row_layout.add_child(Child::clone(c)); // Child is a wrapper around Arc so cloning
                // is cheap
            });
            layout.add_child(row_layout);
        });
        layout
    }

    fn with_children(children: Vec<Vec<Child<I>>>) -> Self {
        Self {
            view: Self::make_view(&children),
            children,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Given Vec<Vec<_>> is not a full grid")]
pub struct GridError;

impl<I: View> GridLayout<I> {
    pub fn from_2d_vec(children: Grid<I>) -> Result<Self, GridError> {
        let columns = children.iter().map(|c| c.len()).max().unwrap_or(0);
        for child in children.iter() {
            if child.len() != columns {
                return Err(GridError);
            }
        }

        let children = children
            .into_iter()
            .map(|row| row.into_iter().map(|c| Child::new(c.into())).collect())
            .collect();

        Ok(Self::with_children(children))
    }
    pub fn replace(&mut self, x: usize, y: usize, new_item: I) -> Option<I> {
        let mut item = self.children.get(y)?.get(x)?.0.inner.write().unwrap();
        Some(std::mem::replace(&mut item, new_item))
    }
}

impl<I: View + Default> GridLayout<I> {
    pub fn from_2d_vec_fill_default(mut children: Vec<Vec<I>>) -> Self {
        let columns = children.iter().map(|c| c.len()).max().unwrap_or(0);
        children
            .iter_mut()
            .for_each(|c| c.resize_with(columns, Default::default));

        let children = children
            .into_iter()
            .map(|row| row.into_iter().map(|c| Child::new(c.into())).collect())
            .collect();

        Self::with_children(children)
    }
    pub fn empty_with_size(rows: usize, columns: usize) -> Self {
        let mut children = Vec::with_capacity(rows);
        children.resize_with(rows, || {
            let mut child_row = Vec::with_capacity(columns);
            child_row.resize_with(columns, Default::default);
            child_row
        });
        Self::with_children(children)
    }
}

impl<I: View> ViewWrapper for GridLayout<I> {
    cursive::wrap_impl!(self.view: LinearLayout);
}
