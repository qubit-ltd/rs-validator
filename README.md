# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` provides type-safe validation traits, explicit binding, structured outcomes, and immutable validator registries for Rust applications.

## Installation

```toml
[dependencies]
qubit-validator = "0.1"
```

## Quick Start

The following program defines a typed `NonBlank` rule, maps its domain error to
the stable code `text.blank`, registers it locally, binds it, and checks both a
valid and an invalid value. It is the complete source of
[`examples/local_registry.rs`](examples/local_registry.rs); run it with
`cargo run --example local_registry`.

```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

#[derive(Debug)]
struct BlankText;

impl fmt::Display for BlankText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("text must not be blank")
    }
}

impl Error for BlankText {}

struct NonBlank;

impl Validator<str> for NonBlank {
    type Error = BlankText;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(BlankText)
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_non_blank,
)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[cfg(feature = "inventory")]
register_validator!(id = "text.non_blank.global", descriptor = &DESCRIPTOR);

static DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("minimum", InputType::Text, false),
    DependencySpec::new("maximum", InputType::Text, false),
];
static DEPENDENCY_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    DEPENDENCIES,
    prepare_non_blank,
)];
static DEPENDENCY_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(DEPENDENCY_SIGNATURES);

fn assert_dependency_order_is_checked() {
    let declared = [DEPENDENCIES[1], DEPENDENCIES[0]];
    let error = DEPENDENCY_DESCRIPTOR
        .bind(ValidatorId::new("text.dependent"), 0, &[], &declared)
        .expect_err("swapped dependency slots must fail during binding");
    assert_eq!(error.kind(), BindErrorKind::DependencyOrderMismatch);
}

fn assert_parameters_are_consumed_once() -> Result<(), BindError> {
    let args = [NamedValidationArgument::new(
        "limit",
        ValidationArgument::Unsigned(10),
    )];
    let mut reader = ArgumentReader::new(&args)?;
    assert_eq!(reader.required_u32("limit")?, 10);
    let error = reader
        .required_u32("limit")
        .expect_err("a parameter cannot be read twice");
    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;

    assert!(registry.get("text.non_blank").is_some());
    let validator = registry.bind("text.non_blank", InputType::Text, &[], &[])?;
    let context = BoundValidationContext::new(&[]);

    let valid = validator.validate(ValidationValue::Text("Ada"), &context)?;
    assert_eq!(valid, ValidationOutcome::Valid);

    let invalid = validator.validate(ValidationValue::Text("   "), &context)?;
    let ValidationOutcome::Invalid(violations) = invalid else {
        panic!("blank text must be invalid");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code(), ViolationCode::new("text.blank"));

    assert_dependency_order_is_checked();
    assert_parameters_are_consumed_once()?;

    #[cfg(feature = "inventory")]
    {
        let global = ValidatorRegistry::try_global()?;
        assert!(global.get("text.non_blank.global").is_some());
    }

    Ok(())
}
```

## Why This Project Exists

Application validation often starts as direct calls and later needs runtime
selection, configuration parameters, dependency values, and diagnostics. This
crate keeps those needs on one contract: rules remain ordinary typed Rust,
while explicit adapters provide a safe type-erased boundary for configured
execution. Invalid business values remain distinct from failures in binding or
execution.

## What It Provides

- `Validator<T, C>` for direct typed validation with domain-specific errors.
- Prepared and bound validators for reusable configured execution.
- Static signatures and descriptors for input and dependency contracts.
- Deterministic local registries keyed by stable validator IDs.
- Structured violations, skipped outcomes, paths, parameters, and bounded
  reports.
- Safe type erasure without `unsafe` and without cloning input values.

## Features

The default feature set is empty. Direct validation, descriptors,
registrations, and local `ValidatorRegistry` construction are available without
features. Enable `inventory` only for process-wide registration through
`register_validator!` and `ValidatorRegistry::try_global`. Prefer a local
registry for isolated tests, plugin boundaries, or multiple rule sets.

Dependencies are declared and supplied as ordered slots. Their order, input
type, and optionality must match the selected signature exactly. Validator
parameters are decoded once by `ArgumentReader`, which rejects duplicates,
unknown names, wrong types, lossy conversions, and repeated consumption.

## Limitations

The crate does not discover object properties, compile model paths, traverse
object graphs, or schedule groups of validators. Callers select values and
dependencies, invoke bound validators, and assemble reports.

Execution is synchronous and borrowed. Errors and their public formatting do
not carry or print the original input. If trusted presentation code needs a
path string, it must explicitly call `ValidationPath::render`.

## Learn More

- [User Guide](doc/user_guide.md)
- [Design](doc/design.md)
- [API documentation](https://docs.rs/qubit-validator)
- [中文文档](README.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-validator](https://github.com/qubit-ltd/rs-validator)
