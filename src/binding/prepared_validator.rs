//! Type-erased prepared validator instances.

use std::sync::Arc;

use super::BindError;
use super::BoundValidationContext;
use super::ExecutionError;
use super::RuleOutcome;
use super::ValidationValue;
use crate::NamedValidationArgument;

/// A configured, immutable validator instance safe to share between calls.
pub trait PreparedValidator: Send + Sync {
    /// Validates one erased input with its ordered dependency context.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError>;
}

/// Constructs an owned prepared validator from declaration parameters.
pub type PrepareFn = fn(&[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError>;
