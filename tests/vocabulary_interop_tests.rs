// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_vocabulary::NamedValidationArgument;
use qubit_validation_vocabulary::ValidationArgument;
use qubit_validator::ArgumentReader;

#[test]
fn argument_reader_accepts_the_shared_vocabulary_type_directly() {
    let arguments = [NamedValidationArgument::new("minimum", ValidationArgument::Unsigned(7))];
    let mut reader = ArgumentReader::new(&arguments).expect("one unique shared argument");
    assert_eq!(reader.required_u32("minimum").expect("u32 value"), 7);
    reader.finish().expect("argument was consumed");
}
