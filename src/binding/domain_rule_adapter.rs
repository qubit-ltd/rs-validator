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
use super::ExecutionError;
use super::ExecutionErrorKind;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::ViolationDraft;
use super::prepare_text_with_context;
use super::prepare_typed_with_context;

/// Prepares a text domain validator with dependency context and error mapping.
///
/// `call` returns the domain result and invocation-specific metadata. The
/// metadata is passed to `map_error` only when the domain result is an error.
/// Infrastructure failures returned directly by `call` are forwarded.
///
/// # Type Parameters
///
/// - `V`: Owned domain validator value used by each invocation.
/// - `E`: Domain error mapped to safe violation drafts.
/// - `D`: Invocation metadata borrowed by the mapper after a domain failure.
/// - `Call`: Operation that invokes the validator with text and context.
/// - `Map`: Mapper from a domain error and metadata to violation drafts.
///
/// # Parameters
///
/// - `dependencies`: Static dependency slots consumed by `call`.
/// - `validator`: Domain rule value borrowed by `call` for each validation.
/// - `call`: Operation separating infrastructure failure from domain result.
/// - `map_error`: Converts a domain failure to a non-empty list of safe
///   violations.
///
/// # Returns
///
/// A shared prepared validator that accepts text and exposes declared
/// dependency slots through the context passed to `call`. When invoked through
/// `BoundValidator`, dependency shape is checked before `call`; direct callers
/// of the prepared validator must supply a context suitable for the closure.
///
/// # Runtime Errors
///
/// An empty invalid draft list becomes `AdapterContractViolation` when the
/// returned validator runs.
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
    Map: Fn(E, &D) -> Vec<ViolationDraft> + Send + Sync + 'static,
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
/// # Type Parameters
///
/// - `T`: Exact `'static` input type accepted by the domain validator.
/// - `V`: Owned domain validator value used by each invocation.
/// - `E`: Domain error mapped to safe violation drafts.
/// - `D`: Invocation metadata borrowed by the mapper after a domain failure.
/// - `Call`: Operation that invokes the validator with `T` and context.
/// - `Map`: Mapper from a domain error and metadata to violation drafts.
///
/// # Parameters
///
/// - `dependencies`: Static dependency slots consumed by `call`.
/// - `validator`: Domain rule value borrowed by `call` for each validation.
/// - `call`: Operation separating infrastructure failure from domain result.
/// - `map_error`: Converts a domain failure to a non-empty list of safe
///   violations.
///
/// # Returns
///
/// A shared prepared validator that accepts `T` and exposes declared
/// dependency slots through the context passed to `call`. When invoked through
/// `BoundValidator`, dependency shape is checked before `call`; direct callers
/// of the prepared validator must supply a context suitable for the closure.
///
/// # Runtime Errors
///
/// An empty invalid draft list becomes `AdapterContractViolation` when the
/// returned validator runs.
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
    Map: Fn(E, &D) -> Vec<ViolationDraft> + Send + Sync + 'static,
{
    prepare_typed_with_context::<T, _>(dependencies, move |input, context| {
        map_domain_result(call(&validator, input, context)?, &map_error)
    })
}

/// Maps a domain call result into the prepared validator protocol.
///
/// Successful domain results are valid. Failed results are passed with their
/// invocation metadata to the mapper; an empty invalid draft list is rejected
/// as an adapter contract violation.
fn map_domain_result<E, D>(
    result: (Result<(), E>, D),
    map_error: &impl Fn(E, &D) -> Vec<ViolationDraft>,
) -> Result<PreparedOutcome, ExecutionError> {
    match result.0 {
        Ok(()) => Ok(PreparedOutcome::Valid),
        Err(error) => PreparedOutcome::invalid(map_error(error, &result.1))
            .map_err(|_| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)),
    }
}
