// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Configured validator occurrences.

use std::fmt;
use std::sync::Arc;

use super::BindError;
use super::BindErrorKind;
use super::BoundValidationContext;
use super::DependencySpec;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::InputType;
use super::PreparedValidator;
use super::ValidationOutcome;
use super::ValidationValue;
use super::ValidatorSignature;
use crate::NamedValidationDependency;
use crate::ValidationPath;
use crate::ValidatorId;

/// A bound occurrence that owns a reusable prepared validator instance.
///
/// A prepared validator reports only whether its execution was valid or
/// invalid. Callers decide whether an occurrence should be skipped and build
/// the corresponding [`ValidationOutcome`] themselves.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use qubit_validator::ArgumentReader;
/// use qubit_validator::BindError;
/// use qubit_validator::BoundValidationContext;
/// use qubit_validator::ExecutionError;
/// use qubit_validator::InputType;
/// use qubit_validator::PreparedOutcome;
/// use qubit_validator::PreparedValidator;
/// use qubit_validator::ValidationOutcome;
/// use qubit_validator::ValidationValue;
/// use qubit_validator::ValidatorDescriptor;
/// use qubit_validator::ValidatorId;
/// use qubit_validator::ValidatorSignature;
///
/// struct AcceptAll;
///
/// impl PreparedValidator for AcceptAll {
///     fn input_type(&self) -> InputType { InputType::Text }
///     fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] { &[] }
///     fn validate(
///         &self,
///         _: ValidationValue<'_>,
///         _: &BoundValidationContext<'_>,
///     ) -> Result<PreparedOutcome, ExecutionError> {
///         Ok(PreparedOutcome::Valid)
///     }
/// }
///
/// fn prepare(
///     _reader: &mut ArgumentReader<'_>,
/// ) -> Result<Arc<dyn PreparedValidator>, BindError> {
///     Ok(Arc::new(AcceptAll))
/// }
///
/// static SIGNATURES: &[ValidatorSignature] = &[
///     ValidatorSignature::new(InputType::Text, &[], prepare),
/// ];
/// let descriptor = ValidatorDescriptor::new(SIGNATURES);
/// let bound = descriptor.bind(ValidatorId::new("example.accept_all"), 0, &[])?;
/// let outcome = bound.validate(
///     ValidationValue::Text("accepted"),
///     &BoundValidationContext::new(&[]),
/// )?;
/// assert_eq!(outcome, ValidationOutcome::Valid);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct BoundValidator {
    /// Immutable prepared implementation shared by cloned bound occurrences.
    prepared: Arc<dyn PreparedValidator>,
    /// Erased input shape accepted by the prepared implementation.
    input: InputType,
    /// Ordered dependency slots required by the prepared implementation.
    dependencies: &'static [DependencySpec],
    /// Stable identifier attached to outcomes and errors.
    rule_id: ValidatorId,
}

impl BoundValidator {
    /// Creates a zero-dependency binding for an already prepared typed rule.
    ///
    /// The binding checks that calls supply exactly `T` and no dependency
    /// slots before invoking the prepared validator. It does not perform
    /// registry lookup or parameter decoding.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Exact concrete value type accepted by the prepared validator.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: Stable identity attached to violations and execution
    ///   errors.
    /// - `prepared`: Immutable prepared implementation shared by clones.
    ///
    /// # Returns
    ///
    /// A reusable bound validator accepting values of type `T` without
    /// dependencies.
    ///
    /// # Errors
    ///
    /// Returns `PreparedSignatureMismatch` with `rule_id` attached when the
    /// prepared validator does not accept exactly `T` or declares dependencies.
    pub fn try_from_prepared<T: 'static>(
        rule_id: ValidatorId,
        prepared: Arc<dyn PreparedValidator>,
    ) -> Result<Self, BindError> {
        if prepared.input_type() != InputType::of::<T>() || !prepared.dependency_specs().is_empty() {
            return Err(BindError::new(BindErrorKind::PreparedSignatureMismatch).with_rule(rule_id));
        }
        Ok(Self {
            prepared,
            input: InputType::of::<T>(),
            dependencies: &[],
            rule_id,
        })
    }

    /// Creates a bound occurrence from a prepared implementation and signature.
    pub(crate) fn new(
        prepared: Arc<dyn PreparedValidator>,
        signature: ValidatorSignature,
        rule_id: ValidatorId,
    ) -> Self {
        Self {
            prepared,
            input: signature.input(),
            dependencies: signature.dependencies(),
            rule_id,
        }
    }

    /// Validates one value after checking its erased input and dependencies.
    ///
    /// Dependency values and paths must already follow
    /// [`Self::dependency_specs`] order. Use [`Self::validate_named`] for
    /// directly assembled dependency sets or whenever same-typed dependencies
    /// could be supplied in the wrong order.
    ///
    /// # Parameters
    /// - `value`: Borrowed target value.
    /// - `context`: Dependency values and paths in signature order.
    ///
    /// # Returns
    /// The bound validation outcome.
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
        outcome
            .into_bound(self.rule_id)
            .map_err(|_| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_rule(self.rule_id))
    }

    /// Validates a value after resolving dependency values by signature name.
    ///
    /// This entry point protects direct callers from swapping dependencies
    /// that have the same input type. Values are reordered into signature slot
    /// order for this call. Use [`Self::validate`] when a caller has already
    /// checked and constructed values in that order. Zero or one dependency
    /// needs no reordering container; two to eight use fixed-size stack
    /// buffers, while more than eight allocate vectors as needed. Cloning a
    /// supplied non-root path may still allocate.
    ///
    /// # Parameters
    /// - `value`: Borrowed target value.
    /// - `dependencies`: Borrowed dependency values keyed by signature name.
    ///   Every declared slot must be present; optional values can use
    ///   [`ValidationValue::Missing`].
    ///
    /// # Returns
    /// The bound validation outcome.
    ///
    /// # Errors
    ///
    /// Returns a named-binding, input-shape, dependency-shape, adapter, or rule
    /// execution error.
    pub fn validate_named(
        &self,
        value: ValidationValue<'_>,
        dependencies: &[NamedValidationDependency<'_>],
    ) -> Result<ValidationOutcome, ExecutionError> {
        if self.dependencies.len() <= 1 {
            for (index, dependency) in dependencies.iter().enumerate() {
                if dependencies[..index]
                    .iter()
                    .any(|previous| previous.name == dependency.name)
                {
                    return Err(ExecutionError::new(ExecutionErrorKind::DuplicateDependencyBinding)
                        .with_rule(self.rule_id)
                        .with_dependency(dependency.name));
                }
                if !self.dependencies.iter().any(|spec| spec.name() == dependency.name) {
                    return Err(ExecutionError::new(ExecutionErrorKind::UnknownDependencyBinding)
                        .with_rule(self.rule_id)
                        .with_dependency(dependency.name));
                }
            }
        }

        if self.dependencies.is_empty() {
            return self.validate(value, &BoundValidationContext::new(&[]));
        }
        if self.dependencies.len() == 1 {
            let spec = self.dependencies[0];
            let dependency = dependencies
                .iter()
                .find(|dependency| dependency.name == spec.name())
                .ok_or_else(|| {
                    ExecutionError::new(ExecutionErrorKind::MissingDependencyBinding)
                        .with_rule(self.rule_id)
                        .with_dependency(spec.name())
                })?;
            let values = [dependency.value];
            return match dependency.path {
                Some(path) => {
                    let paths = [path.clone()];
                    let context = BoundValidationContext::new_with_paths(&values, &paths)
                        .map_err(|error| error.with_rule(self.rule_id))?;
                    self.validate(value, &context)
                }
                None => self.validate(value, &BoundValidationContext::new(&values)),
            };
        }

        self.validate_named_many(value, dependencies)
    }

    /// Resolves multiple named dependencies into signature order before
    /// validation.
    ///
    /// Small dependency sets use fixed-size stack buffers; larger sets allocate
    /// vectors. Missing, duplicate, and unknown bindings are reported before
    /// the prepared validator runs.
    fn validate_named_many(
        &self,
        value: ValidationValue<'_>,
        dependencies: &[NamedValidationDependency<'_>],
    ) -> Result<ValidationOutcome, ExecutionError> {
        const INLINE_SLOTS: usize = 8;
        let mut inline_slots: [Option<NamedValidationDependency<'_>>; INLINE_SLOTS] = [None; INLINE_SLOTS];
        let mut heap_slots = Vec::new();
        let slots: &mut [Option<NamedValidationDependency<'_>>] = if self.dependencies.len() <= INLINE_SLOTS {
            &mut inline_slots[..self.dependencies.len()]
        } else {
            heap_slots.resize(self.dependencies.len(), None);
            heap_slots.as_mut_slice()
        };
        let mut has_path = false;
        for dependency in dependencies.iter().copied() {
            let Some(index) = self.dependencies.iter().position(|spec| spec.name() == dependency.name) else {
                return Err(ExecutionError::new(ExecutionErrorKind::UnknownDependencyBinding)
                    .with_rule(self.rule_id)
                    .with_dependency(dependency.name));
            };
            if slots[index].replace(dependency).is_some() {
                return Err(ExecutionError::new(ExecutionErrorKind::DuplicateDependencyBinding)
                    .with_rule(self.rule_id)
                    .with_dependency(dependency.name));
            }
            has_path |= dependency.path.is_some();
        }
        for (index, slot) in slots.iter().enumerate() {
            if slot.is_none() {
                return Err(ExecutionError::new(ExecutionErrorKind::MissingDependencyBinding)
                    .with_rule(self.rule_id)
                    .with_dependency(self.dependencies[index].name()));
            }
        }
        if self.dependencies.len() <= INLINE_SLOTS {
            let mut values = [ValidationValue::Missing; INLINE_SLOTS];
            for (index, slot) in slots.iter().enumerate() {
                values[index] = slot.as_ref().expect("checked slot").value;
            }
            let values = &values[..self.dependencies.len()];
            if !has_path {
                return self.validate(value, &BoundValidationContext::new(values));
            }
            let mut paths: [ValidationPath; INLINE_SLOTS] = std::array::from_fn(|_| ValidationPath::root());
            for (index, slot) in slots.iter().enumerate() {
                if let Some(path) = slot.as_ref().expect("checked slot").path {
                    paths[index] = path.clone();
                }
            }
            let context = BoundValidationContext::new_with_paths(values, &paths[..self.dependencies.len()])
                .map_err(|error| error.with_rule(self.rule_id))?;
            return self.validate(value, &context);
        }
        let values: Vec<_> = slots
            .iter()
            .map(|slot| slot.as_ref().expect("checked slot").value)
            .collect();
        if !has_path {
            return self.validate(value, &BoundValidationContext::new(&values));
        }
        let paths: Vec<_> = slots
            .iter()
            .map(|slot| {
                slot.as_ref()
                    .expect("checked slot")
                    .path
                    .cloned()
                    .unwrap_or_else(ValidationPath::root)
            })
            .collect();
        let context =
            BoundValidationContext::new_with_paths(&values, &paths).map_err(|error| error.with_rule(self.rule_id))?;
        self.validate(value, &context)
    }

    /// Returns the selected input shape.
    #[must_use]
    #[inline]
    pub const fn input_type(&self) -> InputType {
        self.input
    }

    /// Returns dependencies in their execution slot order.
    #[must_use]
    #[inline]
    pub const fn dependency_specs(&self) -> &'static [super::DependencySpec] {
        self.dependencies
    }

    /// Returns the stable rule identifier.
    #[must_use]
    #[inline]
    pub const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }

    /// Checks the erased input against the selected signature.
    fn check_input(&self, value: ValidationValue<'_>) -> Result<(), ExecutionError> {
        if value.is_missing() || !self.input.accepts(value) {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch).with_rule(self.rule_id));
        }
        Ok(())
    }

    /// Checks dependency values against the selected signature.
    fn check_dependencies(&self, context: &BoundValidationContext<'_>) -> Result<(), ExecutionError> {
        context
            .check_specs(self.dependencies)
            .map_err(|error| error.with_rule(self.rule_id))
    }
}

impl fmt::Debug for BoundValidator {
    /// Formats structural metadata without exposing the prepared
    /// implementation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundValidator")
            .field("rule_id", &self.rule_id)
            .field("input", &self.input)
            .finish_non_exhaustive()
    }
}
