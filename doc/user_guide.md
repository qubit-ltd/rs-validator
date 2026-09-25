# qubit-validator User Guide

[简体中文](user_guide.zh_CN.md)

Applies to `qubit-validator` 0.1.x · Minimum supported Rust: 1.94

## Purpose and Audience

This guide is for Rust library and application authors who need ordinary
typed validation as well as configured rule lookup by stable ID. It covers the
public API without assuming a model framework, code generator, or scheduler.

## Conceptual Model

| Concept | Meaning |
| --- | --- |
| `Validator<T, C>` | Typed rule called directly with a borrowed value and domain context. It returns the rule's domain error. |
| `PreparedValidator` | Reusable, type-erased instance created after configuration is decoded. |
| `BoundValidator` | Prepared instance paired with a stable rule ID and selected signature. It checks target and dependency shapes before execution. |
| `ValidationOutcome` | Result of a completed validation: valid, invalid, or skipped. Invalid data is not an execution error. |
| `ExecutionError` | A shape, dependency, adapter, or external execution failure returned as `Err`. It contains no source error. |
| `ValidationReport` | Caller-owned collection of violations and skipped occurrences, with optional count limits. |

An adapter maps a rule's domain error to a safe `ViolationDraft`. The bound
validator attaches the stable rule ID and returns a final `Violation`.

## Scenario: Validate a Configured Display Name

The example validates a display name with a `NonBlank` rule. It registers the
rule as `text.non_blank`, maps its domain error to `text.blank`, binds it once,
and checks both an accepted and a rejected value. The complete runnable source
is [`examples/local_registry.rs`](../examples/local_registry.rs); run it with:

```bash
cargo run --example local_registry --locked
```

## Installation and Minimal Configuration

Add the crate without optional features for direct validation and local
registries:

```toml
[dependencies]
qubit-validator = "0.1"
```

The default feature set supports local registries. A preparation function
decodes parameters and returns an `Arc<dyn PreparedValidator>`; a static
`ValidatorSignature` and `ValidatorDescriptor` describe the rule. See the
linked example for a complete definition. The call site is:

```rust
let registry = ValidatorRegistry::from_registrations([registration])?;
let validator = registry.bind("text.non_blank", InputType::Text, &[], &[])?;
let context = BoundValidationContext::new(&[]);

let accepted = validator.validate(ValidationValue::Text("Ada"), &context)?;
assert_eq!(accepted, ValidationOutcome::valid());

let rejected = validator.validate(ValidationValue::Text("  "), &context)?;
let mut report = ValidationReport::new();
assert!(report.record_outcome(0, ValidationPath::root(), rejected)?);
assert!(!report.is_valid());
```

## Core Workflow

1. Implement `Validator<T, C>` and call it directly when runtime selection is
   unnecessary.
2. Use `prepare_text_validator` or `prepare_typed_validator` for rules that do
   not consume dependency slots. Their mapper converts a domain error to a
   stable code and safe parameters.
3. Declare the accepted input and ordered dependencies in a
   `ValidatorSignature`. Put signatures in a `ValidatorDescriptor` and
   associate it with a `ValidatorId` and `RegistrationSource`.
4. Build a local `ValidatorRegistry`, bind a rule, and reuse its
   `BoundValidator`. Preparation happens during binding, not on each call.
5. Pass a borrowed `ValidationValue` and `BoundValidationContext` to each
   invocation. Handle non-exhaustive public enums with a fallback arm.
6. Send each returned `ValidationOutcome` to `ValidationReport::record_outcome`
   when the caller needs an aggregate. The crate does not traverse objects or
   schedule rule groups.

## Advanced Usage: Context-Aware Adapters

Use a context-aware adapter when a rule must compare its target with an
already-selected dependency. The signature declares the slot, and the
`BoundValidator` checks its order, shape, and required/optional status before
the adapter calls the typed validator.

```rust
struct MatchesExpected;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesExpected {
    type Error = DependencyMismatch;

    fn validate(
        &self,
        value: &str,
        context: &BoundValidationContext<'a>,
    ) -> Result<(), Self::Error> {
        let expected = context.text(0).map_err(|_| DependencyMismatch)?;
        if value == expected { Ok(()) } else { Err(DependencyMismatch) }
    }
}

let prepared = prepare_contextual_text_validator(MatchesExpected, |_| {
    ViolationDraft::new(ViolationCode::new("text.dependency_mismatch"))
});
```

`DependencyMismatch` is the rule's own `std::error::Error` type. Its message
and the mapper output should not include either text value. For an optional
typed slot, use `context.optional_typed::<T>(index)`; only the explicit
`ValidationValue::Missing` marker becomes `None`. A wrong type remains an
execution error.

Text rules can use `prepare_contextual_text_validator`; typed rules can use
`prepare_contextual_typed_validator::<T, _, _, _>`. The corresponding simple
adapters remain available for rules that do not use context.

## Collecting Outcomes and Prerequisites

`record_outcome` centralizes occurrence ordering and report limits. An invalid
outcome must contain at least one violation. A failed-prerequisite skip must
contain at least one prerequisite violation, which stays nested in the skipped
entry instead of appearing among top-level violations. The occurrence path is
prefixed once to each invalid violation's relative path; a root violation path
uses the occurrence path itself. Prerequisite evidence retains its absolute
path to the original failure. The occurrence path on a skipped outcome identifies
the skipped target. Prepared rules return only `Valid` or `Invalid`; the caller
creates a skipped outcome. Use static declared names with `with_field` and
`MapEntry` for runtime map positions.

```rust
let rule_id = ValidatorId::new("text.required");
let earlier = Violation::new(rule_id, ViolationCode::new("text.blank"));
let mut report = ValidationReport::new();
assert!(report.record_outcome(
    0,
    ValidationPath::root().with_field("password"),
    ValidationOutcome::invalid(vec![earlier])?,
)?);
let earlier = report.violations()[0].clone();
assert!(report.record_outcome(
    1,
    ValidationPath::root().with_field("confirmation"),
    ValidationOutcome::failed_prerequisite(vec![earlier])?,
)?);
assert_eq!(report.violations().len(), 1);
assert_eq!(report.violations()[0].path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.skipped()[0].path(), &ValidationPath::root().with_field("confirmation"));
assert_eq!(report.skipped()[0].prerequisites()[0].path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.failure_count(), 2);
```

The returned `bool` means the complete outcome fit its configured limit; it
does not mean the validation passed. A capacity rejection returns `Ok(false)`
and marks the report truncated. An invalid outcome shape returns
`ValidationOutcomeError` and leaves the report unchanged. `max_violations`
bounds the sum of retained top-level violations and prerequisite evidence.
If no failure capacity remains, a failed-prerequisite skip is not stored with
an empty evidence list. A skip rejected by `max_skipped` consumes no failure
capacity.

## Local and Inventory Registries

`ValidatorRegistry::from_registrations` builds an explicit local registry. It
is deterministic and supports multiple rule sets in one process. Enable
`inventory` only when linked crates should register rules process-wide:

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

Use `ValidatorRegistry::try_global` to handle duplicate IDs and invalid
descriptors. `global` panics for either registry error. The feature changes how
registrations are discovered; it does not change binding or execution rules.

## Errors and Diagnostics

- A domain error from `Validator<T, C>` becomes a `ViolationDraft` through its
  adapter mapper.
- `BindError` reports configuration problems such as missing rules, unsupported
  input shapes, malformed parameters, or dependency declaration mismatches.
- `ValidatorRegistryError` reports duplicate IDs and invalid descriptors.
- `ExecutionError` reports target/dependency shape, adapter contract, or
  external execution failures. Its `Display`, `Debug`, and standard error chain
  do not expose or retain an underlying source error.
- `ValidationOutcome::Invalid` is a completed validation result. It is not an
  `ExecutionError`.

`ArgumentReader` consumes a present parameter on its first typed read, even if
conversion fails. A second read returns `ParameterAlreadyConsumed`. After
reading supported parameters, call `finish` to reject unconsumed names.

`ValidationPath` remains structured. `Display` and `Debug` avoid exposing
field names and map positions; trusted presentation code must explicitly call
`ValidationPath::render` when disclosure is appropriate. Keep rejected input
out of errors and `ViolationParam` values.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `MissingRule` | Confirm the stable ID is in the selected local registry. For global discovery, enable `inventory` and link the registering crate. |
| `UnsupportedInput` or `InputTypeMismatch` | Use the exact `InputType` declared by the selected signature. No implicit text-to-typed conversion occurs. |
| Dependency declaration error | Compare the supplied declarations with the signature; order, shape, and optionality must match. |
| Missing dependency during execution | Check that every required slot has a value and that optional absence uses `ValidationValue::Missing`. |
| `UnknownParameter` | Read supported values and then call `ArgumentReader::finish`. |
| `ParameterAlreadyConsumed` | Decode each parameter once and store the result in the prepared validator. |
| `AdapterContractViolation` | Check custom `PreparedValidator` output. Invalid outcomes need violations; `PreparedOutcome` has only valid and invalid states. |
| `Ok(false)` from `record_outcome` | A report limit rejected part or all of this outcome; inspect `is_truncated()` and configure limits for the expected workload. |

## Limitations and Best Practices

- The crate does not discover fields, traverse object graphs, compile model
  paths, or schedule multiple rules. Keep those policies in the caller.
- Keep validator IDs and violation codes stable for downstream consumers.
- Prefer local registries unless process-wide discovery is required.
- Treat dependency order as an ABI-like contract; coordinate slot changes with
  every caller and validator implementation.
- Use `ValidationReport::with_limits` for untrusted or large workloads.
  `max_violations` bounds retained top-level violations and prerequisite
  evidence together; `max_skipped` bounds skipped occurrences. Collection
  stops retaining excess evidence and marks the report truncated.
- Validation is synchronous and borrows values for each call. Prepared
  validators require `Send + Sync`; the crate does not create threads or assume
  an async runtime.

## Further Reading

- [Design and invariants](design.md)
- [Runnable local-registry example](../examples/local_registry.rs)
- [API documentation](https://docs.rs/qubit-validator)
- [Project README](../README.md)
- [中文 README](../README.zh_CN.md)
- [简体中文用户指南](user_guide.zh_CN.md)
