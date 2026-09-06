# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` provides type-safe validation traits, structured validation reports, and immutable registries for Qubit applications.

## Installation

```toml
[dependencies]
qubit-validator = "0.1"
```

## Quick Start

Implement the root `Validator<T, C>` trait for a rule with an explicit typed context. Rules that need model metadata can be prepared once as a `PreparedValidator` and assembled into a local `ValidatorRegistry`; direct validation remains a normal typed Rust call.

```rust,ignore
use qubit_validator::Validator;
use std::convert::Infallible;

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = Infallible;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        let _ = value;
        Ok(())
    }
}
```

## Capabilities

- Typed validators with domain-specific errors.
- Borrowed parameters and dependency values.
- Safe type-erased invocation without `unsafe`.
- Structured `Violation` and `ValidationReport` values with safe paths and parameters.
- Deterministic local registries; the optional `inventory` feature supplies process-wide discovery.

## Features

The default feature set is empty. The typed descriptor, registration, and local registry APIs are always available; `registry` is retained as a compatibility feature for downstream manifests. Enable `inventory` when process-wide registration through `register_validator!` is required. Applications that need isolated tests or multiple rule sets should build a local `ValidatorRegistry` instead.

## Limitations

The crate does not discover model properties or schedule validation. `qubit-model-metadata` owns model path compilation and `ValidationPlan` execution. This crate only supplies the typed rule contract, binding primitives, reports, and registries. Runtime failures and value violations are represented separately, so a missing rule or unsupported feature cannot be mistaken for valid input.

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
