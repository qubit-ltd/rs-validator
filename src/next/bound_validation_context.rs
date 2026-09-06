//! Ordered, shape-checked dependency values for one validation call.

use super::DependencySpec;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::ValidationPath;
use super::ValidationValue;

/// Borrowed dependency slots used during a synchronous validation call.
pub struct BoundValidationContext<'a> {
    values: &'a [ValidationValue<'a>],
    paths: Option<&'a [ValidationPath]>,
}

impl<'a> BoundValidationContext<'a> {
    /// Creates a context with root paths for each dependency slot.
    #[must_use]
    pub fn new(values: &'a [ValidationValue<'a>]) -> Self {
        Self {
            values,
            paths: None,
        }
    }

    /// Creates a context with model-provided relative dependency paths.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error when the two slices differ in length.
    pub fn new_with_paths(
        values: &'a [ValidationValue<'a>],
        paths: &'a [ValidationPath],
    ) -> Result<Self, ExecutionError> {
        if values.len() != paths.len() {
            return Err(ExecutionError::new(
                ExecutionErrorKind::AdapterContractViolation,
            ));
        }
        Ok(Self {
            values,
            paths: Some(paths),
        })
    }

    /// Returns the value at an ordered dependency slot.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error for an invalid slot index.
    pub fn value(&self, index: usize) -> Result<ValidationValue<'a>, ExecutionError> {
        self.values
            .get(index)
            .copied()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation))
    }

    /// Returns the path at an ordered dependency slot.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract error for an invalid slot index.
    pub fn dependency_path(&self, index: usize) -> Result<&ValidationPath, ExecutionError> {
        match self.paths {
            Some(paths) => paths
                .get(index)
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)),
            None if index < self.values.len() => Ok(&ROOT_PATH),
            None => Err(ExecutionError::new(
                ExecutionErrorKind::AdapterContractViolation,
            )),
        }
    }

    /// Downcasts a required dependency to its exact type.
    ///
    /// # Errors
    ///
    /// Returns a shape, missing-value, or slot error.
    pub fn typed<T: 'static>(&self, index: usize) -> Result<&'a T, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Typed(value) => value
                .downcast_ref::<T>()
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
            ValidationValue::Missing => Err(ExecutionError::new(
                ExecutionErrorKind::MissingRequiredDependencyValue,
            )),
            ValidationValue::Text(_) => Err(ExecutionError::new(
                ExecutionErrorKind::DependencyTypeMismatch,
            )),
        }
    }

    /// Downcasts an optional dependency, treating only `Missing` as `None`.
    ///
    /// # Errors
    ///
    /// Returns a shape or slot error. A wrong concrete type is never treated
    /// as an absent optional value.
    pub fn optional_typed<T: 'static>(
        &self,
        index: usize,
    ) -> Result<Option<&'a T>, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Missing => Ok(None),
            ValidationValue::Typed(value) => value
                .downcast_ref::<T>()
                .map(Some)
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch)),
            ValidationValue::Text(_) => Err(ExecutionError::new(
                ExecutionErrorKind::DependencyTypeMismatch,
            )),
        }
    }

    /// Borrows a required text dependency.
    ///
    /// # Errors
    ///
    /// Returns a shape, missing-value, or slot error.
    pub fn text(&self, index: usize) -> Result<&'a str, ExecutionError> {
        match self.value(index)? {
            ValidationValue::Text(value) => Ok(value),
            ValidationValue::Missing => Err(ExecutionError::new(
                ExecutionErrorKind::MissingRequiredDependencyValue,
            )),
            ValidationValue::Typed(_) => Err(ExecutionError::new(
                ExecutionErrorKind::DependencyTypeMismatch,
            )),
        }
    }

    pub(crate) fn check_specs(&self, specs: &[DependencySpec]) -> Result<(), ExecutionError> {
        if self.values.len() != specs.len()
            || self.paths.is_some_and(|paths| paths.len() != specs.len())
        {
            return Err(ExecutionError::new(
                ExecutionErrorKind::AdapterContractViolation,
            ));
        }
        for (value, spec) in self.values.iter().copied().zip(specs.iter().copied()) {
            if value.is_missing() {
                if !spec.optional() {
                    return Err(ExecutionError::new(
                        ExecutionErrorKind::MissingRequiredDependencyValue,
                    ));
                }
                continue;
            }
            if !spec.input().accepts(value) {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::DependencyTypeMismatch,
                ));
            }
        }
        Ok(())
    }
}

static ROOT_PATH: ValidationPath = ValidationPath::root();

impl std::fmt::Debug for BoundValidationContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundValidationContext")
            .field("slot_count", &self.values.len())
            .finish()
    }
}
