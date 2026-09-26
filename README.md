# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validator` gives Rust rules a typed validation contract and provides an explicit route to configured execution by stable rule ID. It helps library and application authors avoid mixing invalid business values with binding and execution failures while keeping rule selection and reporting under caller control.

## Installation

```toml
[dependencies]
qubit-validator = "0.1"
```

## Quick Start

For example, a form handler can bind a registered `NonBlank` rule once and
reuse it for each display name. The complete runnable
[`examples/local_registry.rs`](examples/local_registry.rs) defines the rule,
registers it, checks accepted and rejected values, and asserts the resulting
violation code:

```bash
cargo run --example local_registry --locked
```

The example confirms that `"Ada"` is valid and that blank text produces the
`text.blank` violation. It also demonstrates dependency-order and parameter
consumption errors; with `inventory` enabled, it exercises process-wide
registration.

## Why This Project Exists

Applications often begin with direct calls to validation functions, then need
runtime rule selection, configuration, ordered dependencies, and structured
diagnostics. This crate keeps rules as ordinary typed Rust code and adds a
small, explicit binding boundary when configured execution is needed.

## What It Provides

- `Validator<T, C>` for direct validation with domain-specific errors.
- Prepared and bound validators for reusable configured execution.
- Static signatures and descriptors for input and dependency contracts.
- Deterministic local registries keyed by stable validator IDs.
- Structured violations, skipped outcomes, paths, safe parameters, and bounded
  reports assembled through `ValidationReport::record_outcome`.
- Context-aware text and typed adapters for rules that read ordered dependency
  slots.

The crate does not discover object properties, traverse object graphs, compile
model paths, schedule rule groups, or localize messages. Callers choose values,
invoke validators, and decide how to present diagnostics.

## Features and Safety Boundaries

The default feature set is empty. Direct validation, adapters, descriptors,
and local `ValidatorRegistry` values work without optional features. Enable
`inventory` only when process-wide registration through `register_validator!`
and `ValidatorRegistry::try_global` is needed.

Dependencies are declared as ordered slots on each signature. Binding a selected signature copies its dependency specifications directly into the bound validator; model metadata validates actual dependency declarations and execution checks runtime slot shape. `ExecutionError` retains an owned cause only for explicit trusted diagnostics through `trusted_source()`. Ordinary formatting and `Error::source()` remain redacted. Violation parameters must not contain rejected input. Failed-prerequisite skips reference retained violations by opaque `FailureId`, so the original failure is counted and rendered once. `record_outcome` returns a `RecordedOutcome` with completion status and IDs retained from that occurrence. Failure references do not consume `max_violations`; `max_skipped` limits skipped occurrences. `ValidationReport::failures()` iterates original violations only. Parameter `Debug` output redacts names and values.

## Learn More

- [User Guide](doc/user_guide.md)
- [Design](doc/design.md)
- [API documentation](https://docs.rs/qubit-validator)
- [中文 README](README.zh_CN.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- [中文设计文档](doc/design.zh_CN.md)

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
