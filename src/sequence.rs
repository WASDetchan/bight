#[derive(thiserror::Error)]
pub enum SequenceMatchError {
    CanBeContined{hint: String},
    CannotBeContined,
}

// struct SequenceBinding<T>
