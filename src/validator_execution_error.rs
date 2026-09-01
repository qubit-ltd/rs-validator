//! Validator execution errors.

use std::any::TypeId;

use thiserror::Error;

/// Failure produced by a type-erased validator invocation.
#[derive(Debug, Error)]
pub enum ValidatorExecutionError {
    /// The supplied value does not match the registered value type.
    #[error("validator for {expected_type} received an incompatible value type {actual_type:?}")]
    TypeMismatch {
        /// Expected Rust value type name.
        expected_type: &'static str,
        /// Actual process-local Rust type identity.
        actual_type: TypeId,
    },
    /// The typed validator rejected the supplied value.
    #[error("validator {validator_type} rejected the value: {source}")]
    ValidationFailed {
        /// Rust validator type name.
        validator_type: &'static str,
        /// Validator-specific source error.
        #[source]
        source: Box<dyn std::error::Error>,
    },
}
