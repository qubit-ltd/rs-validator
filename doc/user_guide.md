# qubit-validator user guide

[中文版本](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-validator)

This guide describes `qubit-validator` 0.1.0 for Rust 1.94 or later. It is for application and library authors who need reusable validation rules, structured results, or a registry that can bind rules by stable identifier.

## Purpose and audience

The crate owns the validation boundary, not model traversal. Use it when a caller already has a value and needs to run a typed rule, prepare a rule for repeated erased calls, or collect violations in a bounded report. Model metadata and validation-plan scheduling remain outside this crate.

## Conceptual model

| Object | Role |
| --- | --- |
| `Validator<T, C>` | A normal typed rule over a borrowed value and immutable context. |
| `PreparedValidator` | A shareable, type-erased rule instance. |
| `ValidatorSignature` and `ValidatorDescriptor` | The input shape, dependency slots, and preparation function accepted by a rule definition. |
| `ValidatorRegistry` | A deterministic collection of registrations that can bind a rule by ID and input shape. |
| `ValidationOutcome` | Valid, invalid with `Violation` values, or explicitly skipped. |
| `ValidationReport` | A bounded aggregate of violations and skipped occurrences. |

There are two error boundaries: binding errors (`BindError`) mean a definition or call could not be prepared; execution errors (`ExecutionError`) mean a prepared rule could not run with the supplied value or dependencies. A rule violation is data returned through `ValidationOutcome::Invalid`, not an execution failure.

## Scenario: reject a blank user name

The success criterion is that a non-blank name returns `Valid`, while whitespace-only input returns a violation with the stable code `user.name.blank`.

## Installation and minimal configuration

```toml
[dependencies]
qubit-validator = "0.1"
```

The default feature set is empty. Enable `inventory` only when the application wants process-wide registration through `register_validator!`:

```toml
qubit-validator = { version = "0.1", features = ["inventory"] }
```

## Core workflow

### 1. Write and use a typed rule

Implement `Validator<T>` when the caller already knows the concrete input type. The default context is `()`; use `Validator<T, C>` when the rule needs an immutable typed context.

```rust
use qubit_validator::Validator;

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

assert!(NonBlank.validate("Ada", &()).is_ok());
assert!(NonBlank.validate("  ", &()).is_err());
```

### 2. Adapt the rule for an erased boundary

`prepare_text_validator` adapts a `Validator<str>` and maps its domain error to a `ViolationDraft`. The prepared rule borrows the input at execution time and can be shared through `Arc`.

```rust
use qubit_validator::BoundValidationContext;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;

let prepared = prepare_text_validator(NonBlank, |_| {
    ViolationDraft::new(ViolationCode::new("user.name.blank"))
});
let outcome = prepared
    .validate(ValidationValue::Text("  "), &BoundValidationContext::new(&[]))?;
assert!(matches!(outcome, PreparedOutcome::Invalid(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a concrete `T: 'static`, use `prepare_typed_validator` and pass `ValidationValue::Typed(&value)` at execution time.

### 3. Bind through a local registry

For rule lookup by stable ID, declare a `ValidatorSignature`, put it in a static `ValidatorDescriptor`, and create a `ValidatorRegistration` with a `ValidatorId` and `RegistrationSource`. Build an isolated registry with `ValidatorRegistry::from_registrations`. It sorts registrations by ID and rejects duplicate IDs or invalid descriptors.

Call `registry.bind(id, input_type, params, dependencies)` once per configured rule occurrence, then reuse the returned `BoundValidator` for calls to `validate`. A descriptor may expose multiple signatures, but each input shape must be unique.

The `inventory` feature adds `register_validator!` and `ValidatorRegistry::try_global()`/`global()` for process-wide discovery. Prefer local registries when tests or tenants need independent rule sets.

## Advanced usage

- Declare ordered `DependencySpec` values for values supplied through `BoundValidationContext`. Required slots reject `ValidationValue::Missing`; optional slots accept it and can be read with `optional_typed`.
- Use `NamedValidationArgument` and `ValidationArgument` for borrowed, domain-neutral preparation parameters.
- Return `PreparedOutcome::Skipped` with `SkipReason::MissingOptional` or `SkipReason::FailedPrerequisite` when the executor deliberately does not run a rule.
- Build nested locations with `ValidationPath::root().with_field("profile").with_index(0)`. A `ViolationDraft` or `Violation` can carry that path and static `ViolationParam` values.
- Use `ValidationReport::with_limits` when a caller must bound retained violations or skipped entries. Reaching a limit marks the report truncated.

## Errors and diagnostics

- `ValidatorRegistryError` reports duplicate IDs and invalid descriptors while building a registry.
- `BindError` reports missing rules, unsupported or ambiguous signatures, dependency declaration mismatches, and preparation failures.
- `ExecutionError` reports input-shape mismatches, missing or incorrectly typed dependencies, adapter contract violations, and rule execution failures.
- `ValidationOutcome::Invalid` contains structured `Violation` values. `ValidationOutcome::Skipped` preserves the reason and prerequisite violations.

Do not treat a binding or execution error as a successful validation result. Use the error kind accessors and the associated rule/dependency information in diagnostics.

## Troubleshooting

1. If binding reports `UnsupportedInput`, compare the requested `InputType` with the descriptor signatures.
2. If binding reports a dependency error, compare names, order, input shapes, and optionality with the declared `DependencySpec` values.
3. If execution reports a type mismatch, ensure `ValidationValue` and every dependency slot use the exact declared shape.
4. If a report is not valid, inspect `violations()`, `skipped()`, and `is_truncated()`; a truncated report is not exhaustive.
5. If the global registry fails to initialize, inspect duplicate registration IDs and use `try_global()` to retain the structured error.

## Limitations and best practices

`qubit-validator` does not traverse models, compile property paths, schedule rules, or provide a built-in set of domain rules. It intentionally supports only borrowed text and exact `TypeId`-based typed values at the erased boundary. Prepared validators must be `Send + Sync`; their implementations should therefore keep shared state immutable or synchronize it explicitly. The crate does not promise that a truncated report contains every violation.

## Further reading

- [README](../README.md) · [中文 README](../README.zh_CN.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-validator)
