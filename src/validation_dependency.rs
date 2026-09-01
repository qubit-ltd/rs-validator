//! Validation dependency values.

use std::any::Any;

/// One dependency value addressed by a normalized property path.
#[derive(Clone, Copy)]
pub struct ValidationDependency<'a> {
    path: &'a str,
    value: &'a dyn Any,
}

impl<'a> ValidationDependency<'a> {
    /// Creates one dependency entry.
    #[must_use]
    pub const fn new(path: &'a str, value: &'a dyn Any) -> Self {
        Self { path, value }
    }

    /// Returns the normalized dependency path.
    #[must_use]
    pub const fn path(&self) -> &'a str {
        self.path
    }

    /// Returns the type-erased dependency value.
    #[must_use]
    pub const fn value(&self) -> &'a dyn Any {
        self.value
    }
}
