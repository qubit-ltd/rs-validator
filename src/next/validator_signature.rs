//! Input and dependency signatures for validator definitions.

use super::DependencySpec;
use super::InputType;
use super::PrepareFn;

/// One statically declared way to bind a validator.
#[derive(Clone, Copy)]
pub struct ValidatorSignature {
    input: InputType,
    dependencies: &'static [DependencySpec],
    prepare: PrepareFn,
}

impl ValidatorSignature {
    /// Creates a signature with ordered dependency slots.
    #[must_use]
    pub const fn new(input: InputType, dependencies: &'static [DependencySpec], prepare: PrepareFn) -> Self {
        Self {
            input,
            dependencies,
            prepare,
        }
    }

    /// Returns the accepted value shape.
    #[must_use]
    pub const fn input(self) -> InputType {
        self.input
    }

    /// Returns dependency declarations in slot order.
    #[must_use]
    pub const fn dependencies(self) -> &'static [DependencySpec] {
        self.dependencies
    }

    /// Returns the parameter binding function.
    #[must_use]
    pub const fn prepare(self) -> PrepareFn {
        self.prepare
    }

    pub(crate) fn same_shape(self, other: Self) -> bool {
        self.input == other.input
            && self.dependencies.len() == other.dependencies.len()
            && self
                .dependencies
                .iter()
                .zip(other.dependencies.iter())
                .all(|(left, right)| left == right)
    }
}

impl std::fmt::Debug for ValidatorSignature {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatorSignature")
            .field("input", &self.input)
            .field("dependency_count", &self.dependencies.len())
            .finish()
    }
}
