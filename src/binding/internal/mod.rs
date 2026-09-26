// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private implementations of erased validator adapters.

mod contextual_fn_validator_adapter;
mod contextual_text_validator_adapter;
mod contextual_typed_validator_adapter;
mod text_validator_adapter;
mod typed_validator_adapter;

pub(super) use contextual_fn_validator_adapter::TextContextFnAdapter;
pub(super) use contextual_fn_validator_adapter::TypedContextFnAdapter;
pub(super) use contextual_text_validator_adapter::ContextualTextValidatorAdapter;
pub(super) use contextual_typed_validator_adapter::ContextualTypedValidatorAdapter;
pub(super) use text_validator_adapter::TextValidatorAdapter;
pub(super) use typed_validator_adapter::TypedValidatorAdapter;
