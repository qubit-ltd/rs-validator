//! Configured validator occurrences.

use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::InputType;
use super::PreparedValidator;
use super::RuleOutcome;
use super::ValidationValue;
use super::ValidatorSignature;
use crate::ValidatorId;

/// A bound occurrence that owns a reusable prepared validator instance.
#[derive(Clone)]
pub struct BoundValidator {
    prepared: Arc<dyn PreparedValidator>,
    signature: ValidatorSignature,
    rule_id: Option<ValidatorId>,
}

impl BoundValidator {
    pub(crate) fn new(
        prepared: Arc<dyn PreparedValidator>,
        signature: ValidatorSignature,
        rule_id: Option<ValidatorId>,
    ) -> Self {
        Self {
            prepared,
            signature,
            rule_id,
        }
    }

    pub(crate) fn with_rule(mut self, rule_id: ValidatorId) -> Self {
        self.rule_id = Some(rule_id);
        self
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
    ) -> Result<RuleOutcome, ExecutionError> {
        self.check_input(value)?;
        self.check_dependencies(context)?;
        match self
            .prepared
            .validate(value, context)
            .map_err(|error| error.with_rule_opt(self.rule_id))?
        {
            RuleOutcome::Invalid(issues) if issues.is_empty() => {
                Err(self.contract_error())
            }
            RuleOutcome::Skipped {
                reason,
                ref prerequisites,
            } if matches!(reason, super::SkipReason::MissingOptional)
                && !prerequisites.is_empty() =>
            {
                Err(self.contract_error())
            }
            RuleOutcome::Skipped {
                reason,
                ref prerequisites,
            } if matches!(reason, super::SkipReason::FailedPrerequisite)
                && prerequisites.is_empty() =>
            {
                Err(self.contract_error())
            }
            outcome => Ok(outcome),
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

    /// Returns the rule identifier when bound through a registry.
    #[must_use]
    pub const fn rule_id(&self) -> Option<ValidatorId> {
        self.rule_id
    }

    fn check_input(
        &self,
        value: ValidationValue<'_>,
    ) -> Result<(), ExecutionError> {
        if value.is_missing() || !self.signature.input().accepts(value) {
            return Err(ExecutionError::new(
                ExecutionErrorKind::InputTypeMismatch,
            )
            .with_rule_opt(self.rule_id));
        }
        Ok(())
    }

    fn check_dependencies(
        &self,
        context: &BoundValidationContext<'_>,
    ) -> Result<(), ExecutionError> {
        context
            .check_specs(self.signature.dependencies())
            .map_err(|error| {
                if error.kind() == ExecutionErrorKind::AdapterContractViolation
                {
                    error
                } else {
                    error.with_rule_opt(self.rule_id)
                }
            })
    }

    fn contract_error(&self) -> ExecutionError {
        ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
            .with_rule_opt(self.rule_id)
    }
}

trait ExecutionErrorExt {
    fn with_rule_opt(self, rule_id: Option<ValidatorId>) -> Self;
}

impl ExecutionErrorExt for ExecutionError {
    fn with_rule_opt(self, rule_id: Option<ValidatorId>) -> Self {
        match rule_id {
            Some(rule_id) => self.with_rule(rule_id),
            None => self,
        }
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
