use std::{
    fmt::Debug,
    sync::{Arc, RwLock, TryLockError},
};

use cursive::{View, view::ViewWrapper, views::LinearLayout};

#[derive(Default)]
pub struct Child<I> {
    inner: RwLock<I>,
}

impl<I> From<I> for  Child<I> {
    fn from(value: I) -> Self {
        Self {
            inner: RwLock::new(value)
        }
    }
}

impl<I> Child<I> {
    fn try_read(&self) -> Option<&I> {

        match self.inner.try_read() {
            Err(e) => match &e {
                TryLockError::WouldBlock => None,
                TryLockError::Poisoned(_) => panic!("Child View Lock poisoned: {}", e),
            },
            Ok(v) => Some(v),
        }
    }

    fn try_write(&self) -> Option<&mut I> {
        match self.inner.try_write() {
            Err(e) => match &e {
                TryLockError::Poisoned(_) => panic!("Child View Lock poisoned: {}", e),
                TryLockError::WouldBlock => None,
            },
            Ok( v) => Some( v),
        }
    }
}

impl<I: View> ViewWrapper for Child<I> {
    type V = I;
    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        self.try_read().map(|v| f(v))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        self.try_write.map(|v| f(v))
    }
}

/// Layout that places its children on a grid
#[derive(Default)]
pub struct GridLayout<I: View> {
    children: Vec<Vec<Arc<Child<I>>>>,
    view: LinearLayout,
}

impl<I: Debug> Debug for GridLayout<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GridLayout: Children: {:?}", self.children)
    }
}

impl<I: View> GridLayout<I> {
    fn make_view(children: &[Vec<Arc<Child<I>>>]) -> LinearLayout {
        let mut layout = LinearLayout::vertical();
        children.iter().for_each(|row| {
            let mut row_layout = LinearLayout::horizontal();
            row.iter().for_each(|c| {
                row_layout.add_child(Arc::clone(c));
            });
            layout.add_child(row_layout);
        });
    }

    fn with_children(children: Vec<Vec<Arc<Child<I>>>>) -> Self{
        Self {
            view: Self::make_view(&children),
            children
        }
    }
}

impl<I: View + Default> GridLayout<I> {
    pub fn from_2d_vec(mut children: Vec<Vec<I>>) -> Self {
        let rows = children.len();
        let columns = children.iter().map(|c| c.len()).max().unwrap_or(0);
        children
            .iter_mut()
            .for_each(|c| c.resize_with(columns, Default::default));

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

// impl<I: View> ViewWrapper for GridLayout<I> {
//     type V = LinearLayout;
//     fn with_view<F, R>(&self, f: F) -> Option<R>
//     where
//         F: FnOnce(&Self::V) -> R,
//     {
//     }
// }
