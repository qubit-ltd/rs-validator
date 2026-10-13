# Qubit Validator (`rs-validator`)

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` supplies reusable typed rules, rule binding, and structured validation reports for Rust applications. Consider a `POST /users` endpoint: the server must reject a blank username or a malformed email before writing to the user store. This crate is where those checks live as `Validator` implementations with stable IDs. Field annotations on the domain type only *name* those IDs; `qubit-model-metadata` binds the registered rules and walks the object. Keeping the rules in one registry avoids copying the same checks into every handler.

## A user registration flow

The frontend sends a username and email. The REST handler parses the request into `User` and calls the registration service. At startup, the service registers two custom rules—`app.non_blank` and `app.email`—and builds a reusable validation plan from the `User` field declarations. For each request, the plan runs those rules: a blank username or non-ASCII-email shape returns field paths and stable codes without writing to the user repository. A valid object reaches the repository. The HTTP layer can map rejection to a client error and storage or validation execution failure to a server error; HTTP routing and persistence are application code.

## Installation

For direct rules and local registries alone, `qubit-validator = "0.1"` is enough. The Quick Start also applies those rules through field annotations, so it needs `qubit-model-metadata` with its `validation` feature and `qubit-model-derive`:

```toml
[dependencies]
qubit-validator = "0.1"
qubit-model-metadata = { version = "0.2", features = ["validation"] }
qubit-model-derive = "0.2"
qubit-reflect = "0.2"
```

Rust 1.94 or newer is required.

## Quick start

The program below is a complete, runnable version of that registration decision. Put it in `src/main.rs` of the application described above. It shows the three steps that belong to this crate's public surface: implement `Validator`, register the rules under stable IDs, then reference those IDs from `User` fields. `CountingRepository` only makes the no-write/write branches observable; replace it with an application repository in the REST service.

```rust
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};
use qubit_model_metadata::validation::{ValidationBuildInputs, ValidationOptions, ValidationPlan};
use qubit_reflect::ReflectedRef;
use qubit_validator::{
    ArgumentReader, BindError, InputType, NamedValidationArgument, PreparedValidator,
    RegistrationSource, ValidationReport, Validator, ValidatorDescriptor, ValidatorId,
    ValidatorRegistration, ValidatorRegistry, ValidatorSignature, ViolationCode, ViolationDraft,
    prepare_text_validator,
};

// --- Custom rule: non-blank text -------------------------------------------

#[derive(Debug)]
struct BlankText;
impl fmt::Display for BlankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text must not be blank")
    }
}
impl Error for BlankText {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankText;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
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

static NON_BLANK_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_non_blank)];
static NON_BLANK_DESCRIPTOR: ValidatorDescriptor =
    ValidatorDescriptor::new(NON_BLANK_SIGNATURES);
static NON_BLANK_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.non_blank"),
    &NON_BLANK_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

// --- Custom rule: ASCII email shape ----------------------------------------

#[derive(Debug)]
struct BadEmail;
impl fmt::Display for BadEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text is not a valid ASCII email")
    }
}
impl Error for BadEmail {}

struct AsciiEmail;
impl Validator<str> for AsciiEmail {
    type Error = BadEmail;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(BadEmail)?;
        let domain = parts.next().ok_or(BadEmail)?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || !domain.contains('.')
            || domain.split('.').any(|label| label.is_empty())
        {
            Err(BadEmail)
        } else {
            Ok(())
        }
    }
}

fn prepare_email(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
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

// --- Domain type: annotations name the registered IDs ----------------------

#[Model]
struct User {
    #[validator(id = "app.non_blank")]
    username: String,
    #[validator(id = "app.non_blank")]
    #[validator(id = "app.email")]
    email: String,
}

trait UserRepository {
    fn insert(&self, user: User) -> Result<(), Box<dyn Error>>;
}

enum RegistrationOutcome {
    Created,
    Rejected(ValidationReport),
}

fn register_user(
    plan: &ValidationPlan<'_>,
    repository: &dyn UserRepository,
    request: User,
) -> Result<RegistrationOutcome, Box<dyn Error>> {
    let report = plan.validate(ReflectedRef::new(&request), &ValidationOptions::default())?;
    if report.is_truncated() {
        return Err(std::io::Error::other("registration validation was incomplete").into());
    }
    if !report.is_valid() {
        return Ok(RegistrationOutcome::Rejected(report));
    }
    repository.insert(request)?;
    Ok(RegistrationOutcome::Created)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Build once at application startup; reuse the plan for every request.
    let root = TypeMetadata::of::<User>();
    let roots = [root];
    let models = ModelRegistry::try_global()?;
    let graph = Arc::new(StructureResolver::for_roots(ResolveInputs {
        models: &models,
        roots: &roots,
    }).resolve()?);
    let validators = ValidatorRegistry::from_registrations([
        NON_BLANK_REGISTRATION,
        EMAIL_REGISTRATION,
    ])?;
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph,
        validators: &validators,
    })?;

    struct CountingRepository(Cell<usize>);
    impl UserRepository for CountingRepository {
        fn insert(&self, _user: User) -> Result<(), Box<dyn Error>> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    let repository = CountingRepository(Cell::new(0));
    let invalid = User { username: "   ".into(), email: "wrong".into() };
    let RegistrationOutcome::Rejected(report) = register_user(&plan, &repository, invalid)? else {
        panic!("invalid registration must be rejected");
    };
    assert_eq!(repository.0.get(), 0);
    for violation in report.violations() {
        println!("{}: {}", violation.path().render(), violation.code().as_str());
    }

    let valid = User { username: "ada".into(), email: "ada@example.com".into() };
    assert!(matches!(
        register_user(&plan, &repository, valid)?, RegistrationOutcome::Created
    ));
    assert_eq!(repository.0.get(), 1);
    Ok(())
}
```

Run `cargo run`. The invalid request prints `username: text.blank` and `email: text.email`; the repository count stays at zero. The valid request increments it to one.

`NonBlank` and `AsciiEmail` are ordinary `Validator<str>` implementations. `prepare_text_validator` maps each domain error to a stable violation code; the descriptor and `ValidatorRegistration` publish that prepared rule under `app.non_blank` or `app.email`. The `#[validator(id = "...")]` attributes on `User` only declare which registered IDs apply to each field—they do not run at construction time. `ValidationPlan::build` in `qubit-model-metadata` reads those declarations and binds the matching registrations; `validate` walks the supported fields and returns this crate's `ValidationReport`. Built-in constraints such as `#[text(non_blank)]` can bind without a custom registration; a custom `#[validator(id = "...")]` always needs a matching entry in the registry you pass to the plan.

The service checks `is_truncated()` before accepting or reporting a result. A partial report cannot prove the remaining fields are valid. The email rule checks a simple ASCII shape; it does not verify mailbox ownership. Username availability and other database-dependent policies belong in the application service. Keep rejected input out of diagnostic parameters and logs.

## What it provides

- `Validator<T, C>` for direct typed rules with domain-specific errors.
- `ValidatorSignature`, `ValidatorDescriptor`, and `ValidatorRegistry` for selecting and preparing configured rules by stable ID; `BoundValidator` checks the chosen input and dependency shape.
- `ValidationOutcome`, `Violation`, and `ValidationReport` for separating invalid values from binding or execution failures and collecting caller-visible field results.
- Adapters and `ArgumentReader` for reusable rules with parameters or named dependencies. Optional `inventory` discovers linked custom registrations.

`qubit-validator` alone does not inspect `User` fields or schedule its rules. Direct callers can bind a rule and assemble a report themselves; the Quick Start uses `qubit-model-metadata` as the optional model-level execution path that reads field annotations. See the [validator user guide](doc/user_guide.md) for the lower-level bind-and-report API and the [model validation guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md#core-workflow-validate-a-profile-and-keep-partial-results) for plan construction, supported access shapes, partial reports, and limits.

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
