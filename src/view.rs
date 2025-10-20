use cursive::{view::ViewWrapper, views::LinearLayout};

/// A vertical LinearLayout in which each child is a horizontal LinearLayout
/// Alows access by a pair of indices
pub struct MultiLinearView {
    view: LinearLayout,
}

impl MultiLinearView {
    /// Adds an empty child
    /// Returns the index of the new child
    pub fn add_child(&mut self) -> usize {
        self.view.add_child(LinearLayout::horizontal());
        self.view.len() - 1
    }

    /// Adds given child to self's child with index idx
    /// Returns index of the inserted child in it's horizontal view
    /// Panics
    /// Panics if child with index idx doesn't exist
    pub fn add_child_to_child<V>(&mut self, idx: usize, view: V) -> usize {
        self.view
            .get_child_mut(idx)
            .expect("Tried to add child to a non-existent child");
        1
    }
}

impl ViewWrapper for MultiLinearView {
    cursive::wrap_impl!(self.view: LinearLayout);
}
