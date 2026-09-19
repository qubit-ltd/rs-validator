//! Configured validator occurrences.

use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::InputType;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::ValidationOutcome;
use super::ValidationValue;
use super::ValidatorSignature;
use crate::ValidatorId;
use crate::Violation;

/// A bound occurrence that owns a reusable prepared validator instance.
#[derive(Clone)]
pub struct BoundValidator {
    prepared: Arc<dyn PreparedValidator>,
    signature: ValidatorSignature,
    rule_id: ValidatorId,
}

impl BoundValidator {
    pub(crate) fn new(
        prepared: Arc<dyn PreparedValidator>,
        signature: ValidatorSignature,
        rule_id: ValidatorId,
    ) -> Self {
        Self {
            prepared,
            signature,
            rule_id,
        }
    }

    /// Validates one value after checking its erased input and dependencies.
    ///
    /// # Errors
    ///
    /// Returns a shape, dependency, adapter, or rule execution error.
    pub fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<ValidationOutcome, ExecutionError> {
        self.check_input(value)?;
        self.check_dependencies(context)?;
        let outcome = self
            .prepared
            .validate(value, context)
            .map_err(|error| error.with_rule(self.rule_id))?;
        match outcome {
            PreparedOutcome::Invalid(drafts) if drafts.is_empty() => Err(self.contract_error()),
            PreparedOutcome::Skipped {
                reason,
                ref prerequisites,
            } if matches!(reason, crate::SkipReason::MissingOptional) && !prerequisites.is_empty() => {
                Err(self.contract_error())
            }
            PreparedOutcome::Skipped {
                reason,
                ref prerequisites,
            } if matches!(reason, crate::SkipReason::FailedPrerequisite) && prerequisites.is_empty() => {
                Err(self.contract_error())
            }
            PreparedOutcome::Valid => Ok(ValidationOutcome::Valid),
            PreparedOutcome::Invalid(drafts) => Ok(ValidationOutcome::Invalid(
                drafts
                    .into_iter()
                    .map(|draft| Violation::from_draft(self.rule_id, draft))
                    .collect(),
            )),
            PreparedOutcome::Skipped { reason, prerequisites } => {
                Ok(ValidationOutcome::Skipped { reason, prerequisites })
            }
        }
    }

    /// Returns the selected input shape.
    #[must_use]
    pub const fn input_type(&self) -> InputType {
        self.signature.input()
    }

    /// Returns dependencies in their execution slot order.
    #[must_use]
    pub const fn dependency_specs(&self) -> &'static [super::DependencySpec] {
        self.signature.dependencies()
    }

    /// Returns the stable rule identifier.
    #[must_use]
    pub const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }

    fn check_input(&self, value: ValidationValue<'_>) -> Result<(), ExecutionError> {
        if value.is_missing() || !self.signature.input().accepts(value) {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch).with_rule(self.rule_id));
        }
        Ok(())
    }

    fn check_dependencies(&self, context: &BoundValidationContext<'_>) -> Result<(), ExecutionError> {
        context.check_specs(self.signature.dependencies()).map_err(|error| {
            if error.kind() == ExecutionErrorKind::AdapterContractViolation {
                error
            } else {
                error.with_rule(self.rule_id)
            }
        })
    }

    fn contract_error(&self) -> ExecutionError {
        ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_rule(self.rule_id)
    }
}

impl std::fmt::Debug for BoundValidator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundValidator")
            .field("rule_id", &self.rule_id)
            .field("input", &self.signature.input())
            .finish_non_exhaustive()
    }
}
