# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` gives Rust applications a small, type-safe boundary for reusable validation rules. It keeps direct typed validation, prepared type-erased rules, structured violations, and deterministic rule registries separate so invalid input is not confused with a binding or execution failure.

## Installation

```toml
[dependencies]
qubit-validator = "0.1"
```

## Quick Start

Suppose an application must reject a blank user name and later expose the same rule through a prepared validation boundary. The typed rule can be adapted without stringifying or cloning the input:

```rust,ignore
use qubit_validator::BoundValidationContext;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::Validator;
use qubit_validator::ValidationValue;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;

#[derive(Debug)]
struct BlankName;

impl std::fmt::Display for BlankName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("name is blank")
    }
}

impl std::error::Error for BlankName {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankName;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankName) } else { Ok(()) }
    }
}

let prepared = prepare_text_validator(NonBlank, |_| {
    ViolationDraft::new(ViolationCode::new("user.name.blank"))
});
let outcome = prepared
    .validate(ValidationValue::Text("  "), &BoundValidationContext::new(&[]))?;
assert!(matches!(outcome, PreparedOutcome::Invalid(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The typed implementation remains ordinary Rust code, while the adapter produces a structured violation that a report or registry-driven executor can consume.

## Capabilities

- Typed validators with domain-specific errors.
- Borrowed parameters and dependency values.
- Safe type-erased invocation without `unsafe`.
- Structured `Violation` and `ValidationReport` values with safe paths and parameters.
- Deterministic local registries; the optional `inventory` feature supplies process-wide discovery.

## Features

The default feature set is empty. The typed descriptor, registration, and local registry APIs are always available. Enable `inventory` only when process-wide registration through `register_validator!` is required. Applications that need isolated tests or multiple rule sets should build a local `ValidatorRegistry` instead. Validation reports can be created with `ValidationReport::with_limits` when collection bounds are required.

## Limitations

The crate does not discover model properties or schedule validation. `qubit-model-metadata` owns model path compilation and `ValidationPlan` execution. This crate only supplies the typed rule contract, binding primitives, reports, and registries. Runtime failures and value violations are represented separately, so a missing rule or unsupported feature cannot be mistaken for valid input.

## Learn More

- [English user guide](doc/user_guide.md) / [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-validator)
- [中文 README](README.zh_CN.md)

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
