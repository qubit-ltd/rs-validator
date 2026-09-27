// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapters for domain validators whose errors need domain-specific mapping.

use std::error::Error;
use std::sync::Arc;

use super::BoundValidationContext;
use super::DependencySpec;
use super::DomainErrorDisposition;
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::prepare_text_with_context;
use super::prepare_typed_with_context;

/// Prepares a text domain validator with dependency context and error mapping.
///
/// `call` returns the domain result and invocation-specific metadata. The
/// metadata is passed to `map_error` only when the domain result is an error.
/// Infrastructure failures returned directly by `call` are forwarded.
///
/// # Errors
///
/// An empty invalid draft list becomes `AdapterContractViolation` at runtime.
///
/// # Privacy
///
/// Implementations of `map_error` must map errors to safe codes and parameters;
/// they must not copy raw input or secret error text into a violation.
#[must_use]
pub fn prepare_text_domain_rule<V, E, D, Call, Map>(
    dependencies: &'static [DependencySpec],
    validator: V,
    call: Call,
    map_error: Map,
) -> Arc<dyn PreparedValidator>
where
    V: Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    D: Send + 'static,
    Call: for<'a> Fn(&V, &str, &BoundValidationContext<'a>) -> Result<(Result<(), E>, D), ExecutionError>
        + Send
        + Sync
        + 'static,
    Map: Fn(E, &D) -> DomainErrorDisposition + Send + Sync + 'static,
{
    prepare_text_with_context(dependencies, move |input, context| {
        map_domain_result(call(&validator, input, context)?, &map_error)
    })
}

/// Prepares a typed domain validator with dependency context and error mapping.
///
/// `call` returns the domain result and invocation-specific metadata. The
/// metadata is passed to `map_error` only when the domain result is an error.
/// Infrastructure failures returned directly by `call` are forwarded.
///
/// # Errors
///
/// An empty invalid draft list becomes `AdapterContractViolation` at runtime.
///
/// # Privacy
///
/// Implementations of `map_error` must map errors to safe codes and parameters;
/// they must not copy raw input or secret error text into a violation.
#[must_use]
pub fn prepare_typed_domain_rule<T, V, E, D, Call, Map>(
    dependencies: &'static [DependencySpec],
    validator: V,
    call: Call,
    map_error: Map,
) -> Arc<dyn PreparedValidator>
where
    T: 'static,
    V: Send + Sync + 'static,
    E: Error + Send + Sync + 'static,
    D: Send + 'static,
    Call: for<'a> Fn(&V, &T, &BoundValidationContext<'a>) -> Result<(Result<(), E>, D), ExecutionError>
        + Send
        + Sync
        + 'static,
    Map: Fn(E, &D) -> DomainErrorDisposition + Send + Sync + 'static,
{
    prepare_typed_with_context::<T, _>(dependencies, move |input, context| {
        map_domain_result(call(&validator, input, context)?, &map_error)
    })
}

fn map_domain_result<E, D>(
    result: (Result<(), E>, D),
    map_error: &impl Fn(E, &D) -> DomainErrorDisposition,
) -> Result<PreparedOutcome, ExecutionError> {
    match result.0 {
        Ok(()) => Ok(PreparedOutcome::Valid),
        Err(error) => match map_error(error, &result.1) {
            DomainErrorDisposition::Valid => Ok(PreparedOutcome::Valid),
            DomainErrorDisposition::Invalid(drafts) => PreparedOutcome::invalid(drafts)
                .map_err(|_| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)),
        },
    }
}
