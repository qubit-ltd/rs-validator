//! Safe type-erased validator descriptors.

use std::any::Any;
use std::any::TypeId;

use crate::ValidationContext;
use crate::Validator;
use crate::ValidatorExecutionError;

type ValidateFn = fn(&dyn Any, &ValidationContext<'_>) -> Result<(), ValidatorExecutionError>;

/// An immutable, safely erased validator implementation for one value type.
#[derive(Clone, Copy)]
pub struct ValidatorDescriptor {
    validator_type_id: fn() -> TypeId,
    validator_type_name: fn() -> &'static str,
    value_type_id: fn() -> TypeId,
    value_type_name: fn() -> &'static str,
    validate: ValidateFn,
}

impl ValidatorDescriptor {
    /// Creates a descriptor for validator `V` and value type `T`.
    #[must_use]
    pub const fn of<V, T>() -> Self
    where
        V: Default + Validator<T> + 'static,
        T: 'static,
    {
        Self {
            validator_type_id: TypeId::of::<V>,
            validator_type_name: core::any::type_name::<V>,
            value_type_id: TypeId::of::<T>,
            value_type_name: core::any::type_name::<T>,
            validate: validate::<V, T>,
        }
    }

    /// Returns the process-local validator type identity.
    #[must_use]
    pub fn validator_type_id(&self) -> TypeId {
        (self.validator_type_id)()
    }

    /// Returns the diagnostic validator type name.
    #[must_use]
    pub fn validator_type_name(&self) -> &'static str {
        (self.validator_type_name)()
    }

    /// Returns the process-local value type identity.
    #[must_use]
    pub fn value_type_id(&self) -> TypeId {
        (self.value_type_id)()
    }

    /// Returns the diagnostic value type name.
    #[must_use]
    pub fn value_type_name(&self) -> &'static str {
        (self.value_type_name)()
    }

    /// Invokes the typed validator after checking the erased input type.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch for the wrong value type, or a validation
    /// failure when the typed validator rejects the value.
    pub fn validate(&self, value: &dyn Any, context: &ValidationContext<'_>) -> Result<(), ValidatorExecutionError> {
        (self.validate)(value, context)
    }
}

impl core::fmt::Debug for ValidatorDescriptor {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ValidatorDescriptor")
            .field("validator_type_name", &self.validator_type_name())
            .field("value_type_name", &self.value_type_name())
            .finish_non_exhaustive()
    }
}

/// Downcasts and invokes one typed validator.
fn validate<V, T>(value: &dyn Any, context: &ValidationContext<'_>) -> Result<(), ValidatorExecutionError>
where
    V: Default + Validator<T> + 'static,
    T: 'static,
{
    let value = value
        .downcast_ref::<T>()
        .ok_or_else(|| ValidatorExecutionError::TypeMismatch {
            expected_type: core::any::type_name::<T>(),
            actual_type: value.type_id(),
        })?;
    V::default()
        .validate(value, context)
        .map_err(|source| ValidatorExecutionError::ValidationFailed {
            validator_type: core::any::type_name::<V>(),
            source: Box::new(source),
        })
}
