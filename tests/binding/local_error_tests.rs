// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_contextual_text_validator;
use qubit_validator::prepare_contextual_typed_validator;
use qubit_validator::prepare_text_domain_rule;
use qubit_validator::prepare_text_validator;
use qubit_validator::prepare_typed_domain_rule;
use qubit_validator::prepare_typed_validator;

#[derive(Debug)]
struct LocalError(Rc<u32>);

impl fmt::Display for LocalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("local rejection")
    }
}

impl Error for LocalError {}

struct Reject;

impl<C: ?Sized> Validator<str, C> for Reject {
    type Error = LocalError;

    fn validate(&self, _: &str, _: &C) -> Result<(), Self::Error> {
        Err(LocalError(Rc::new(7)))
    }
}

impl<C: ?Sized> Validator<u32, C> for Reject {
    type Error = LocalError;

    fn validate(&self, _: &u32, _: &C) -> Result<(), Self::Error> {
        Err(LocalError(Rc::new(7)))
    }
}

fn map_error(error: LocalError) -> ViolationDraft {
    assert_eq!(*error.0, 7);
    ViolationDraft::new(ViolationCode::new("test.local_rejected"))
}

#[test]
fn test_all_adapters_accept_local_errors_and_remain_shareable() {
    let adapters: [Arc<dyn PreparedValidator>; 6] = [
        prepare_text_validator(Reject, map_error),
        prepare_typed_validator::<u32, _, _>(Reject, map_error),
        prepare_contextual_text_validator(&[], Reject, map_error),
        prepare_contextual_typed_validator::<u32, _, _, _>(&[], Reject, map_error),
        prepare_text_domain_rule(
            &[],
            Reject,
            |rule, value, _| Ok((rule.validate(value, &()), Rc::new(11_u32))),
            |error, metadata| {
                assert_eq!(**metadata, 11);
                vec![map_error(error)]
            },
        ),
        prepare_typed_domain_rule::<u32, _, _, Rc<u32>, _, _>(
            &[],
            Reject,
            |rule, value, _| Ok((rule.validate(value, &()), Rc::new(13_u32))),
            |error, metadata| {
                assert_eq!(**metadata, 13);
                vec![map_error(error)]
            },
        ),
    ];

    for adapter in adapters {
        let outcome = std::thread::spawn(move || {
            let number = 1_u32;
            let value = if adapter.input_type() == InputType::Text {
                ValidationValue::Text("private input")
            } else {
                ValidationValue::Typed(&number)
            };
            adapter.validate(value, &BoundValidationContext::new(&[]))
        })
        .join()
        .expect("worker completes")
        .expect("domain rejection is an outcome");

        let PreparedOutcome::Invalid(drafts) = outcome else {
            panic!("expected mapped domain rejection");
        };
        assert_eq!(drafts.len(), 1);
        assert_eq!(
            drafts.into_iter().next().expect("one mapped draft"),
            ViolationDraft::new(ViolationCode::new("test.local_rejected")),
        );
    }
}
