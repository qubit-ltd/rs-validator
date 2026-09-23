// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private protocol helpers shared across public identifier types.

mod dotted_identifier;
mod dotted_identifier_error;

pub(crate) use dotted_identifier::validate_dotted_identifier;
pub(crate) use dotted_identifier_error::DottedIdentifierError;
