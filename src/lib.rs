// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Type-safe validation traits and immutable validator registries.
//!
//! Use [`Validator`] for direct typed validation. Use a prepared adapter when
//! a domain rule must cross an erased input boundary. For configured execution,
//! declare [`ValidatorSignature`] values in a [`ValidatorDescriptor`], register
//! the descriptor, then bind it into a reusable [`BoundValidator`]. Aggregate
//! bound results explicitly with [`ValidationReport`] when a caller needs a
//! report; this crate does not schedule rules.
//!
//! A prepared validator is a lower-level entry point. Direct callers must
//! provide a context that satisfies its declared dependency contract; bound
//! execution performs the input and dependency checks before invoking it.
//!
//! # Direct typed validation
//!
//! ```
//! use std::fmt;
//!
//! use qubit_validator::Validator;
//!
//! struct Positive;
//! #[derive(Debug)]
//! struct NotPositive;
//!
//! impl fmt::Display for NotPositive {
//!     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         formatter.write_str("value must be positive")
//!     }
//! }
//! impl std::error::Error for NotPositive {}
//!
//! impl Validator<i32> for Positive {
//!     type Error = NotPositive;
//!
//!     fn validate(&self, value: &i32, _: &()) -> Result<(), Self::Error> {
//!         if *value > 0 { Ok(()) } else { Err(NotPositive) }
//!     }
//! }
//!
//! Positive.validate(&7, &()).unwrap();
//! assert!(Positive.validate(&0, &()).is_err());
//! ```
//!
//! # Configured validation and reporting
//!
//! ```
//! use std::sync::Arc;
//!
//! use qubit_validator::{
//!     BindError, BoundValidationContext, InputType, NamedValidationArgument,
//!     PreparedValidator, RegistrationSource, ValidationOutcome, ValidationPath,
//!     ValidationReport, ValidationValue, Validator, ValidatorDescriptor, ValidatorId,
//!     ValidatorRegistration, ValidatorRegistry, ValidatorSignature, ViolationCode,
//!     prepare_text_validator,
//! };
//!
//! struct NonBlank;
//! #[derive(Debug)]
//! struct Blank;
//! impl std::fmt::Display for Blank {
//!     fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         formatter.write_str("blank name")
//!     }
//! }
//! impl std::error::Error for Blank {}
//!
//! impl Validator<str> for NonBlank {
//!     type Error = Blank;
//!
//!     fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
//!         if value.trim().is_empty() { Err(Blank) } else { Ok(()) }
//!     }
//! }
//!
//! fn prepare(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
//!     Ok(prepare_text_validator(NonBlank, |_| {
//!         qubit_validator::ViolationDraft::new(qubit_validator::ViolationCode::new("text.blank"))
//!     }))
//! }
//!
//! static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare)];
//! static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
//! static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
//!     ValidatorId::new("profile.name.non_blank"),
//!     &DESCRIPTOR,
//!     RegistrationSource::new("profile-rules", "profile", "rules.rs", 1),
//! );
//!
//! let registry = ValidatorRegistry::from_registrations([&REGISTRATION])?;
//! let rule = registry.bind("profile.name.non_blank", InputType::Text, &[])?;
//! let outcome = rule.validate(
//!     ValidationValue::Text("Ada"),
//!     &BoundValidationContext::new(&[]),
//! )?;
//! let mut report = ValidationReport::new();
//! report.record_outcome(0, ValidationPath::root().with_field("name"), outcome)?;
//! assert!(report.is_valid());
//!
//! let invalid = rule.validate(
//!     ValidationValue::Text("  "),
//!     &BoundValidationContext::new(&[]),
//! )?;
//! assert!(matches!(invalid, ValidationOutcome::Invalid(_)));
//! report.record_outcome(
//!     1,
//!     ValidationPath::root().with_field("name"),
//!     invalid,
//! )?;
//! assert!(!report.is_valid());
//! assert_eq!(report.violations()[0].code(), ViolationCode::new("text.blank"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod argument;
mod binding;
mod internal;
mod registry;
mod report;
mod validator;
mod validator_id;
mod validator_id_error;
mod validator_registry_error;

#[cfg(feature = "inventory")]
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use argument::NamedValidationArgument;
pub use argument::ValidationArgument;
pub use binding::ArgumentReader;
pub use binding::BindError;
pub use binding::BindErrorKind;
pub use binding::BoundValidationContext;
pub use binding::BoundValidator;
pub use binding::DependencySpec;
pub use binding::ExecutionError;
pub use binding::ExecutionErrorKind;
pub use binding::InputType;
pub use binding::NamedValidationDependency;
pub use binding::PrepareFn;
pub use binding::PreparedOutcome;
pub use binding::PreparedValidator;
pub use binding::ValidationOutcome;
pub use binding::ValidationValue;
pub use binding::ViolationDraft;
pub use binding::prepare_contextual_text_validator;
pub use binding::prepare_contextual_typed_validator;
pub use binding::prepare_text_domain_rule;
pub use binding::prepare_text_validator;
pub use binding::prepare_text_with_context;
pub use binding::prepare_typed_domain_rule;
pub use binding::prepare_typed_validator;
pub use binding::prepare_typed_with_context;
pub use registry::RegistrationSource;
pub use registry::ValidatorDescriptor;
pub use registry::ValidatorRegistration;
#[cfg(feature = "inventory")]
#[doc(hidden)]
pub use registry::ValidatorRegistrationFactory;
pub use registry::ValidatorRegistry;
pub use registry::ValidatorSignature;
pub use report::FailureId;
pub use report::PathSegment;
pub use report::RecordedOutcome;
pub use report::SkipReason;
pub use report::SkippedValidation;
pub use report::ValidationLimits;
pub use report::ValidationOutcomeError;
pub use report::ValidationPath;
pub use report::ValidationReport;
pub use report::Violation;
pub use report::ViolationCode;
pub use report::ViolationCodeError;
pub use report::ViolationParam;
pub use validator::Validator;
pub use validator_id::ValidatorId;
pub use validator_id_error::ValidatorIdError;
pub use validator_registry_error::ValidatorRegistryError;
