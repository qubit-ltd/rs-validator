# Qubit Validator (`rs-validator`)

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` supplies reusable typed rules, rule binding, and structured validation reports for Rust applications. Consider a `POST /users` endpoint: the server must reject a blank username or a malformed email before writing to the user store. This crate is where those checks live as `Validator` implementations with stable IDs. For applications that use field annotations, those annotations only *name* rule IDs; the optional `qubit-model-metadata` integration binds registered rules and walks the object. Keeping the rules in one registry avoids copying the same checks into every handler.

## A user registration flow

The frontend sends a username and email. The REST handler parses the request into `User` and calls the registration service. At startup, the service registers and binds `app.non_blank` and `app.email`. For each request, it applies the bound rules to the `username` and `email` fields and records their outcomes in a report: invalid input returns field paths and stable codes without a repository write, while a complete valid report allows the write. The HTTP layer can map rejection to a client error and storage or validation execution failure to a server error; HTTP routing and persistence are application code. Apps that declare rules on model fields can use the optional `qubit-model-metadata` integration described below.

## Installation

Rust 1.94 or newer is required. Direct rules and a local registry need only this crate:

<!-- validator-quick-start:manifest -->
```toml
[dependencies]
qubit-validator = "0.1"
```

## Quick start

This complete program follows a user registration flow: register rules, bind them at application startup, and check each field before writing to the repository. `CountingRepository` makes the rejection and write branches observable; a service can replace it with its own repository.

<!-- validator-quick-start:source -->
```rust
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_validator::{
    ArgumentReader, BoundValidationContext, InputType, NamedValidationArgument, PreparedValidator,
    RegistrationSource, ValidationPath, ValidationReport, ValidationValue, Validator,
    ValidatorDescriptor, ValidatorId, ValidatorRegistration, ValidatorRegistry, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

#[derive(Debug)]
struct RuleError(&'static str);
impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for RuleError {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = RuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(RuleError("text is blank"))
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}
static NON_BLANK_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_non_blank,
)];
static NON_BLANK_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(NON_BLANK_SIGNATURES);
static NON_BLANK_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.non_blank"),
    &NON_BLANK_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

struct AsciiEmail;
impl Validator<str> for AsciiEmail {
    type Error = RuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(RuleError("invalid email shape"))?;
        let domain = parts.next().ok_or(RuleError("invalid email shape"))?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || !domain.contains('.')
            || domain.split('.').any(str::is_empty)
        {
            Err(RuleError("invalid ASCII email shape"))
        } else {
            Ok(())
        }
    }
}
fn prepare_email(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(AsciiEmail, |_| {
        ViolationDraft::new(ViolationCode::new("text.email"))
    }))
}
static EMAIL_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_email)];
static EMAIL_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(EMAIL_SIGNATURES);
static EMAIL_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.email"),
    &EMAIL_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

struct User {
    username: String,
    email: String,
}
struct CountingRepository(Cell<usize>);
impl CountingRepository {
    fn insert(&self, _user: &User) {
        self.0.set(self.0.get() + 1);
    }
}

fn validate_and_insert(
    user: &User,
    non_blank: &qubit_validator::BoundValidator,
    email: &qubit_validator::BoundValidator,
    repository: &CountingRepository,
) -> Result<ValidationReport, Box<dyn Error>> {
    let mut report = ValidationReport::new();
    for (index, (field, value, rule)) in [
        ("username", user.username.as_str(), non_blank),
        ("email", user.email.as_str(), email),
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = rule.validate(
            ValidationValue::Text(value),
            &BoundValidationContext::new(&[]),
        )?;
        report.record_outcome(index, ValidationPath::root().with_field(field), outcome)?;
    }
    if !report.is_truncated() && report.is_valid() {
        repository.insert(user);
    }
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Register and bind once during application startup.
    let registry =
        ValidatorRegistry::from_registrations([&NON_BLANK_REGISTRATION, &EMAIL_REGISTRATION])?;
    let non_blank = registry.bind("app.non_blank", InputType::Text, &[])?;
    let email = registry.bind("app.email", InputType::Text, &[])?;
    let repository = CountingRepository(Cell::new(0));

    let invalid = User {
        username: "   ".into(),
        email: "wrong".into(),
    };
    let report = validate_and_insert(&invalid, &non_blank, &email, &repository)?;
    assert!(!report.is_truncated());
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].code().as_str(), "text.blank");
    assert_eq!(report.violations()[0].path().render(), "username");
    assert_eq!(report.violations()[1].code().as_str(), "text.email");
    assert_eq!(report.violations()[1].path().render(), "email");
    assert_eq!(repository.0.get(), 0);

    let valid = User {
        username: "ada".into(),
        email: "ada@example.com".into(),
    };
    let report = validate_and_insert(&valid, &non_blank, &email, &repository)?;
    assert!(!report.is_truncated());
    assert!(report.is_valid());
    assert_eq!(repository.0.get(), 1);
    Ok(())
}
```

Run `cargo run`. The invalid request report contains `username: text.blank` and `email: text.email`, and the repository count stays at zero. The valid request is written once. The email rule checks a demonstration ASCII email shape; it does not verify mailbox ownership.

## Model annotations

`qubit-validator` defines, prepares, registers, and executes rules. Object field traversal is provided by `qubit-model-metadata`. The former `#[Model]` and `ValidationPlan` example belongs to this optional advanced path.

For a checkout with the layout below, an application can use these local dependencies. This example is not required to get started with `qubit-validator`:

```text
<checkout>/rust-common/{rs-validator,rs-model-metadata,rs-reflect,rs-validation-rules}
<checkout>/app
```

In `<checkout>/app/Cargo.toml`:

```toml
[dependencies]
qubit-validator = { path = "../rust-common/rs-validator" }
qubit-model-metadata = { path = "../rust-common/rs-model-metadata", features = ["validation"] }
qubit-model-derive = { path = "../rust-common/rs-model-metadata/derive" }
qubit-reflect = { path = "../rust-common/rs-reflect" }
```

In the current checkout, both `qubit-model-metadata` and `qubit-model-derive` set `publish = false`. See the [model validation user guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md).

## What it provides

- `Validator<T, C>` for direct typed rules with domain-specific errors.
- `ValidatorSignature`, `ValidatorDescriptor`, and `ValidatorRegistry` for selecting and preparing configured rules by stable ID; `BoundValidator` checks the chosen input and dependency shape.
- `ValidationOutcome`, `Violation`, and `ValidationReport` for separating invalid values from binding or execution failures and collecting caller-visible field results.
- Adapters and `ArgumentReader` for reusable rules with parameters or named dependencies. Optional `inventory` discovers linked custom registrations.

`qubit-validator` does not inspect model fields or schedule rules. The Quick Start shows the direct path: bind rules at startup, validate caller-selected fields, and assemble a report before deciding whether to save. Applications that want field annotations and automatic object traversal can use the optional `qubit-model-metadata` integration described in [Model annotations](#model-annotations). See the [validator user guide](doc/user_guide.md) for the bind-and-report API and the [model validation guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md#core-workflow-validate-a-profile-and-keep-partial-results) for plan construction, supported access shapes, partial reports, and limits.

## Learn more

- [Validator user guide](doc/user_guide.md) · [Design](doc/design.md) · [API reference](https://docs.rs/qubit-validator)
- [Model validation guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md#core-workflow-validate-a-profile-and-keep-partial-results) · [Field declaration guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/derive/doc/user_guide.md)
- [中文 README](README.zh_CN.md) · [中文用户手册](doc/user_guide.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
.infra/bin/ci-check.sh

# Check code coverage
.infra/bin/coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines and keep
public API documentation and tests current. This repository uses
`.infra/bin/align-ci.sh` as its formatting command and
`.infra/bin/ci-check.sh` as its final check. Default `cargo fmt --check` may
report differences from the project format; do not apply its full-repository
diff.

## Author

**Haixing Hu**

Repository: [https://github.com/qubit-ltd/rs-validator](https://github.com/qubit-ltd/rs-validator)
