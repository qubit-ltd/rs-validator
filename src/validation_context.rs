//! Validation call-site context.

use std::any::Any;

use crate::NamedValidationArgument;
use crate::ValidationArgument;
use crate::ValidationDependency;

/// Immutable parameters and dependency values supplied to one validation call.
#[derive(Clone, Copy, Default)]
pub struct ValidationContext<'a> {
    arguments: &'a [NamedValidationArgument<'a>],
    dependencies: &'a [ValidationDependency<'a>],
}

impl<'a> ValidationContext<'a> {
    /// Creates a validation context from borrowed entries.
    #[must_use]
    pub const fn new(
        arguments: &'a [NamedValidationArgument<'a>],
        dependencies: &'a [ValidationDependency<'a>],
    ) -> Self {
        Self {
            arguments,
            dependencies,
        }
    }

    /// Returns all arguments in declaration order.
    #[must_use]
    pub const fn arguments(&self) -> &'a [NamedValidationArgument<'a>] {
        self.arguments
    }

    /// Finds an argument by exact name.
    #[must_use]
    pub fn argument(&self, name: &str) -> Option<ValidationArgument<'a>> {
        self.arguments
            .iter()
            .find(|argument| argument.name() == name)
            .map(NamedValidationArgument::value)
    }

    /// Returns all dependency values in declaration order.
    #[must_use]
    pub const fn dependencies(&self) -> &'a [ValidationDependency<'a>] {
        self.dependencies
    }

    /// Finds a dependency by normalized property path.
    #[must_use]
    pub fn dependency(&self, path: &str) -> Option<&'a dyn Any> {
        self.dependencies
            .iter()
            .find(|dependency| dependency.path() == path)
            .map(ValidationDependency::value)
    }
}
