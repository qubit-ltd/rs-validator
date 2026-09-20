# qubit-validator User Guide

## Purpose and Audience

`qubit-validator` is for Rust libraries and applications that need both normal
typed validation and configured lookup by a stable rule ID. This guide assumes
basic familiarity with Rust traits and error handling. It focuses on the public
API; it does not assume a model framework, code generator, or scheduler.

## Conceptual Model

The API deliberately separates six concepts:

| Concept | Meaning |
| --- | --- |
| Direct typed validation | Calling `Validator<T, C>::validate` with concrete Rust types and receiving the rule's domain error. |
| Prepared validator | A reusable, type-erased rule instance produced after configuration parameters have been decoded. It returns `PreparedOutcome` drafts or an `ExecutionError`. |
| Bound validator | A prepared validator paired with a selected signature and stable rule ID. It checks the input and ordered dependency slots before executing. |
| Validation outcome | The result of one successful invocation: `Valid`, `Invalid`, or `Skipped`. Invalid input is data, not an execution failure. |
| Execution error | An infrastructure or contract failure such as an input mismatch, missing required dependency, or adapter contract violation. It is returned as `Err`. |
| Report | A caller-assembled `ValidationReport` that aggregates final `Violation` and `SkippedValidation` values, optionally with limits. |

This separation lets a rule keep a domain-specific error during direct calls,
then map that error to a stable `ViolationDraft` only when it crosses the
prepared boundary.

## Scenario: Validate a Configured Display Name

The running scenario uses a `NonBlank` rule for a display name. Direct typed
validation answers a simple question with `Result<(), BlankText>`. Configured
validation registers the same rule as `text.non_blank`, maps `BlankText` to the
public violation code `text.blank`, and invokes it through a registry.

The complete runnable source is
[`examples/local_registry.rs`](../examples/local_registry.rs). The full source
is reproduced in the next section so it can be copied as a program.

## Installation and Minimal Configuration

Add the crate without optional features for direct validation and local
registries:

```toml
[dependencies]
qubit-validator = "0.1"
```

Local registration is available in the default feature set. The following is
the complete `examples/local_registry.rs` program:

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

Run the canonical source with:

```bash
cargo run --example local_registry --locked
```

## Core Workflow

1. Implement `Validator<T, C>` for a typed rule. Call it directly when runtime
   lookup is unnecessary.
2. Adapt the rule with `prepare_text_validator` or `prepare_typed_validator`.
   The mapper converts a domain error into a `ViolationDraft` with a stable
   code and safe parameters.
3. Put the preparation function in one or more static `ValidatorSignature`
   values. Each signature fixes its accepted input shape and ordered dependency
   slots.
4. Put the signatures in a static `ValidatorDescriptor`, then associate the
   descriptor with a `ValidatorId` and `RegistrationSource`.
5. Freeze registrations into a local registry, look up or bind the rule, then
   reuse the resulting `BoundValidator`.
6. For each call, pass a borrowed `ValidationValue` and a
   `BoundValidationContext`. Handle the non-exhaustive `ValidationOutcome` with
   a fallback arm.
7. If validating several occurrences, convert the outcomes into a
   caller-owned `ValidationReport`. The crate does not schedule or traverse
   those occurrences for you.

Preparation happens during binding, not for every validation call. A
`BoundValidator` owns the prepared instance through `Arc` and can be cloned.

## Advanced Usage: Local and Inventory Registries

Use `ValidatorRegistry::from_registrations` for deterministic, explicit
composition. It is always available, works well in tests, and allows different
registries in the same process.

Process-wide discovery is optional. Enable it explicitly:

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

Then submit a static descriptor and obtain the global registry. Both excerpts
come from `examples/local_registry.rs`:

```rust
#[cfg(feature = "inventory")]
register_validator!(id = "text.non_blank.global", descriptor = &DESCRIPTOR);
```

```rust
#[cfg(feature = "inventory")]
{
    let global = ValidatorRegistry::try_global()?;
    assert!(global.get("text.non_blank.global").is_some());
}
```

Compile and execute those feature-gated paths with:

```bash
cargo run --example local_registry --all-features --locked
```

Duplicate IDs cause registry construction to fail with
`ValidatorRegistryError::DuplicateId`. Prefer `try_global` where that failure
must be handled; `global` panics on an invalid linked registry.

## Dependency Slots

A dependency signature is an ordered list of `DependencySpec` values. Names
make diagnostics understandable, but names do not turn the list into a map:
position, `InputType`, and optionality all form the contract. Binding rejects a
caller declaration that contains the right dependencies in the wrong order.

This focused example comes directly from `examples/local_registry.rs`:

```rust
fn assert_dependency_order_is_checked() {
    let declared = [DEPENDENCIES[1], DEPENDENCIES[0]];
    let error = DEPENDENCY_DESCRIPTOR
        .bind(ValidatorId::new("text.dependent"), 0, &[], &declared)
        .expect_err("swapped dependency slots must fail during binding");
    assert_eq!(error.kind(), BindErrorKind::DependencyOrderMismatch);
}
```

Run that source with:

```bash
cargo run --example local_registry --locked
```

At execution time, `BoundValidationContext` must contain the same number and
shape of values. Use `ValidationValue::Missing` only for an optional slot.
`new_with_paths` can associate each slot with a structured dependency path for
diagnostics.

## Errors and Diagnostics

- A rule's domain error belongs to direct typed validation. An adapter mapper
  turns it into a `ViolationDraft`; the bound validator attaches the rule ID
  and returns final `Violation` values in `ValidationOutcome::Invalid`.
- `BindError` reports configuration failures such as missing rules, unsupported
  inputs, malformed parameters, and dependency declaration mismatches.
- `ValidatorRegistryError` reports duplicate IDs or invalid descriptors while
  freezing a registry.
- `ExecutionError` reports infrastructure and adapter failures. It is separate
  from an invalid business value.
- `ValidationReport` is an aggregate chosen by the caller. It can bound stored
  violations and skipped entries and records truncation.

Parameters are decoded through `ArgumentReader`. A present parameter is
consumed by its first typed read, including a read that fails type or range
conversion. Reading it again returns `ParameterAlreadyConsumed`. This excerpt
comes directly from `examples/local_registry.rs`:

```rust
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
```

Run that source with:

```bash
cargo run --example local_registry --locked
```

Public diagnostics intentionally omit raw inputs and raw parameter values.
`ExecutionError` may retain an internal source error, but its `Display` and
`Debug` output do not expose the source text. `ValidationPath::Display` is also
redacted; trusted presentation code must opt in to `ValidationPath::render`.

## Troubleshooting

- **`MissingRule`**: verify the stable ID and that the registration was included
  in the selected local registry. For a global registry, enable `inventory` and
  ensure the registering crate is linked.
- **`UnsupportedInput` or `InputTypeMismatch`**: bind and invoke using the exact
  `InputType` declared by the selected signature. Text and typed values are not
  implicitly converted.
- **Dependency declaration errors**: compare the supplied dependency slice with
  `BoundValidator::dependency_specs`; order is significant.
- **`UnknownParameter`**: consume every configured parameter, then call
  `ArgumentReader::finish`. Remove misspelled or unsupported names.
- **`ParameterAlreadyConsumed`**: decode each present parameter exactly once and
  store the decoded value in the prepared validator.
- **`AdapterContractViolation`**: check custom `PreparedValidator`
  implementations. An invalid outcome must contain at least one draft, and
  skipped outcomes must satisfy the prerequisite rules documented by the API.

## Limitations and Best Practices

- The crate does not discover fields, traverse objects, compile paths, or
  schedule multiple rules. Keep those policies in the caller.
- Keep validator IDs and violation codes stable. Treat changes as protocol
  changes for downstream consumers.
- Prefer local registries unless process-wide discovery is a real requirement.
- Treat dependency order as an ABI-like contract. Append or reorder slots only
  with coordinated callers and implementations.
- Use the provided text and typed adapters for ordinary `Validator`
  implementations. A custom prepared adapter must uphold the outcome contract.
- Match public non-exhaustive enums with a wildcard arm so minor releases can
  add variants.
- Put only presentation-safe values in `ViolationParam`; never add the rejected
  input to an error message or structured parameter.
- Use `ValidationReport::with_limits` when accepting untrusted or very large
  collections of validation work.

## Further Reading

- [Design and invariants](design.md)
- [Runnable local-registry example](../examples/local_registry.rs)
- [API documentation](https://docs.rs/qubit-validator)
- [Project README](../README.md)
- [中文文档](../README.zh_CN.md)
