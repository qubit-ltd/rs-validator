# qubit-validator User Guide

[简体中文](user_guide.zh_CN.md)

Applies to `qubit-validator` 0.1.x · Minimum supported Rust: 1.94

## Purpose and Audience

This guide is for Rust library and application authors who need ordinary
typed validation as well as configured rule lookup by stable ID. It covers the
public API without assuming a model framework, code generator, or scheduler.

## Scenario: Validate a Configured Display Name

The example validates a display name with a `NonBlank` rule. It registers the
rule as `text.non_blank`, maps its domain error to `text.blank`, binds it once,
and checks both an accepted and a rejected value. The complete runnable source
is [`examples/local_registry.rs`](../examples/local_registry.rs); run it with:

```bash
cargo run --example local_registry --locked
```

The following application flow uses the same `text.non_blank` rule. These
snippets belong to one application module: later snippets use types and
functions declared earlier. The application owns its profile store and request
response.

First, define the typed rule and its domain error:

```rust
use std::{error::Error, fmt};
use qubit_validator::Validator;

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

    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
    }
}
```

Next, define a preparation function and describe the accepted input. This
rule takes no parameters, so `finish()` rejects unexpected configuration:

```rust
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, InputType, NamedValidationArgument,
    PreparedValidator, ValidatorDescriptor, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_non_blank),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
```

At startup, register and bind the rule once, then inject the resulting
`BoundValidator` into the profile service:

```rust
use qubit_validator::{
    BoundValidator, RegistrationSource, ValidatorId,
    ValidatorRegistration, ValidatorRegistry,
};

fn bind_display_name_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    Ok(registry.bind("text.non_blank", InputType::Text, &[])?)
}
```

For each request, record the result at the field path. The caller checks the
report before asking its own store to save the value:

```rust
use qubit_validator::{
    BoundValidationContext, ValidationPath, ValidationReport, ValidationValue,
};

fn check_display_name(
    validator: &BoundValidator,
    display_name: &str,
) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let context = BoundValidationContext::new(&[]);
    let outcome = validator.validate(ValidationValue::Text(display_name), &context)?;
    let mut report = ValidationReport::new();
    report.record_outcome(
        0,
        ValidationPath::root().with_field("display_name"),
        outcome,
    )?;
    Ok(report)
}

trait ProfileStore {
    fn save_display_name(
        &self,
        user_id: &str,
        display_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

enum UpdateProfileResult {
    Updated,
    Rejected(ValidationReport),
}

fn update_profile(
    store: &dyn ProfileStore,
    validator: &BoundValidator,
    user_id: &str,
    incoming_display_name: &str,
) -> Result<UpdateProfileResult, Box<dyn std::error::Error>> {
    let report = check_display_name(validator, incoming_display_name)?;
    if !report.is_valid() {
        return Ok(UpdateProfileResult::Rejected(report));
    }
    store.save_display_name(user_id, incoming_display_name)?;
    Ok(UpdateProfileResult::Updated)
}
```

`"Ada"` reaches the store and returns `Updated`; a blank name returns
`Rejected(report)` without calling the store. Store failures remain application
errors. An HTTP handler can map the violation code and controlled field path to
an API response without including the incoming value in messages or logs.

## Conceptual Model

| Concept | Meaning |
| --- | --- |
| `Validator<T, C>` | Typed rule called directly with a borrowed value and domain context. It returns the rule's domain error. |
| `PreparedValidator` | Reusable, type-erased instance created after configuration is decoded. |
| `BoundValidator` | Prepared instance paired with a stable rule ID and selected signature. It checks target and dependency shapes before execution. |
| `ValidationOutcome` | Result of a completed validation: valid, invalid, or skipped. Invalid data is not an execution error. |
| `ExecutionError` | A shape, dependency, adapter, or external execution failure returned as `Err`. It retains an owned cause for explicit trusted diagnostics through `trusted_source()`; `Display`, `Debug`, and `Error::source()` do not expose it. |
| `ValidationReport` | Caller-owned collection of violations and skipped occurrences, with optional count limits. |

An adapter maps a rule's domain error to a safe `ViolationDraft`. The bound
validator attaches the stable rule ID and returns a final `Violation`.

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
let validator = registry.bind("text.non_blank", InputType::Text, &[])?;
let context = BoundValidationContext::new(&[]);

let accepted = validator.validate(ValidationValue::Text("Ada"), &context)?;
assert_eq!(accepted, ValidationOutcome::valid());

let rejected = validator.validate(ValidationValue::Text("  "), &context)?;
let mut report = ValidationReport::new();
assert!(report.record_outcome(0, ValidationPath::root(), rejected)?.complete());
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

`ViolationParam` is a restricted vocabulary for diagnostic values, not proof
that a value is safe or trusted. `Unsigned(minimum)` from rule configuration is
allowed; `Unsigned(rejected_number)` from rejected input is not. A `Token`'s
`'static` lifetime does not prove its source. Callers must not include rejected
input or sensitive values derived from it.

For an already prepared rule with no dependencies, use
`BoundValidator::try_from_prepared<T>`. It skips registry and parameter preparation and returns `BindError` unless the prepared validator accepts exactly `T` and has no dependencies.

## Advanced Usage: Context-Aware Adapters

Use a context-aware adapter when a rule must compare its target with an
already-selected dependency. The signature declares the slot, and the
`BoundValidator` checks its shape and required/optional status before the
adapter calls the typed validator. The prepared adapter receives the same static dependency slice as the signature. `validate` accepts dependencies in the
signature's declared order. Direct callers can use `validate_named` to bind
each value by its declared name; this prevents same-typed slots from being
silently swapped. The named entry point allocates temporary reorder buffers.

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

let prepared = prepare_contextual_text_validator(DEPENDENCIES, MatchesExpected, |_| {
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

Use the closure adapters when a rule needs multiple violation drafts or must
distinguish invalid data from an execution failure. The closure returns a
`PreparedOutcome` for validation results and `ExecutionError` for execution
failures:

```rust
let prepared = prepare_text_with_context(DEPENDENCIES, |value, context| {
    let expected = context.text(0)?;
    if value == expected {
        Ok(PreparedOutcome::Valid)
    } else {
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(
            ViolationCode::new("text.dependency_mismatch"),
        )]))
    }
});
```

The adapter checks that the target is text. `BoundValidator` checks declared
dependency count, types, optionality, and paths before invoking the closure.
Keep raw input out of violations and public error formatting.

## Collecting Outcomes and Prerequisites

`record_outcome` requires successful calls to use non-decreasing occurrence numbers. Reusing the same number is allowed when one position produces multiple results. A lower number returns `ValidationOutcomeError::OutOfOrderOccurrence` without changing the report. Results are appended in call order so issued `FailureId` indices remain stable. An invalid outcome must contain at least one violation. A failed-prerequisite skip carries one or more opaque `FailureId` references to violations already retained by the same report. References are validated before mutation and do not consume `max_violations`; each original violation is counted once. `max_skipped` limits skipped occurrences. `record_outcome` returns a `RecordedOutcome`, whose `complete()` reports whether all of the outcome fit and whose `failure_ids()` identifies retained failures from that occurrence. `ValidationReport::failures()` iterates original violations only. `failure(id)` resolves a reference to its violation. `trusted_source()` is the explicit diagnostic entry point for an owned execution cause; ordinary error formatting and `Error::source()` remain redacted.
```rust
let rule_id = ValidatorId::new("text.required");
let earlier = Violation::new(rule_id, ViolationCode::new("text.blank"));
let mut report = ValidationReport::new();
let original = report.record_outcome(
    0,
    ValidationPath::root().with_field("password"),
    ValidationOutcome::invalid(vec![earlier])?,
)?;
assert!(original.complete());
let failure_id = original.failure_ids()[0];
let skipped = report.record_outcome(
    1,
    ValidationPath::root().with_field("confirmation"),
    ValidationOutcome::failed_prerequisite(vec![failure_id])?,
)?;
assert!(skipped.complete());
assert_eq!(report.violations().len(), 1);
assert_eq!(report.failure(failure_id).unwrap().path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.skipped()[0].path(), &ValidationPath::root().with_field("confirmation"));
assert_eq!(report.skipped()[0].prerequisites(), &[failure_id]);
assert_eq!(report.failure_count(), 1);
assert_eq!(report.failures().count(), report.failure_count());
```

`RecordedOutcome::complete()` reports whether the outcome fit its configured limits; it does not report whether validation passed. A capacity rejection returns an incomplete receipt and marks the report truncated. An invalid outcome shape returns `ValidationOutcomeError` and leaves the report unchanged. `max_violations` bounds retained original violations.
Failed-prerequisite skips must reference one or more original violations
already retained by this report. Those references remain valid even when
`max_violations` is full and consume no additional violation capacity;
`max_skipped` independently limits retained skip entries. A skip rejected by
`max_skipped` marks the report truncated and returns an incomplete receipt.

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
  external execution failures. Its `Display`, `Debug`, and `Error::source()`
  keep the owned cause redacted. Trusted diagnostic code may inspect it through
  the explicit `trusted_source()` accessor.
- `ValidationOutcome::Invalid` is a completed validation result. It is not an
  `ExecutionError`.

`ArgumentReader` consumes a present parameter on its first typed read, even if
conversion fails. A second read returns `ParameterAlreadyConsumed`. After
reading supported parameters, call `finish` to reject unconsumed names.
For an optional string setting, `optional_str("minimum")` returns `None` when
absent and borrows the supplied string when present; a type error consumes the
parameter just like other typed reads.

`ValidationPath` remains structured. `Display` and `Debug` avoid exposing
field names and map positions; trusted presentation code must explicitly call
`ValidationPath::render` when disclosure is appropriate. Keep rejected input
out of errors and `ViolationParam` values; the type restricts representation,
but callers remain responsible for provenance and redaction.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `MissingRule` | Confirm the stable ID is in the selected local registry. For global discovery, enable `inventory` and link the registering crate. |
| `UnsupportedInput` or `InputTypeMismatch` | Use the exact `InputType` declared by the selected signature. No implicit text-to-typed conversion occurs. |
| Invalid signature declaration | Check that each signature has unique, non-empty dependency names and a unique input shape. Model metadata checks actual declared paths against the selected signature. |
| Missing dependency during execution | Check that every required slot has a value and that optional absence uses `ValidationValue::Missing`. |
| `UnknownParameter` | Read supported values and then call `ArgumentReader::finish`. |
| `ParameterAlreadyConsumed` | Decode each parameter once and store the result in the prepared validator. |
| `AdapterContractViolation` | Check custom `PreparedValidator` output. Invalid outcomes need violations; `PreparedOutcome` has only valid and invalid states. |
| `!recorded.complete()` | A report limit rejected part or all of this outcome; inspect `is_truncated()` and configure limits for the expected workload. If the caller stops validation early, call `mark_truncated()` so the report is identified as incomplete. |

## Limitations and Best Practices

- The crate does not discover fields, traverse object graphs, compile model
  paths, or schedule multiple rules. Keep those policies in the caller.
- Keep validator IDs and violation codes stable for downstream consumers.
- Prefer local registries unless process-wide discovery is required.
- Treat dependency order as an ABI-like contract; coordinate slot changes with
  every caller and validator implementation.
- Use `ValidationReport::with_limits` for untrusted or large workloads.
  `max_violations` bounds retained original violations; references from skips
  consume no additional failure capacity. `max_skipped` bounds skipped
  occurrences. These limits do not bound input size, validation work, path
  size, prerequisite-reference list length, or memory already allocated for
  outcomes. Enforce those budgets at the application boundary. Collection
  stops retaining excess entries and marks the report truncated.
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
