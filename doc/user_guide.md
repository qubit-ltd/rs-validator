# Qubit Validator user guide

[README](../README.md) · [简体中文](user_guide.zh_CN.md) · [API reference](https://docs.rs/qubit-validator)

This guide covers `qubit-validator` 0.1.x on Rust 1.94 or newer. It is for developers adding typed validation to a Rust application, especially when several request paths select rules from configuration. Follow [Integrate a profile update](#integrate-a-profile-update) and [Check the result before saving](#check-the-result-before-saving) for the basic path. The remaining sections answer specific questions about dependencies, parameters, reports, registration, and failures.

## Contents

- [The problem it solves](#the-problem-it-solves)
- [Integrate a profile update](#integrate-a-profile-update)
- [Check the result before saving](#check-the-result-before-saving)
- [Compare two profile fields](#compare-two-profile-fields)
- [Configure a rule once at startup](#configure-a-rule-once-at-startup)
- [Collect several checks and handle limits](#collect-several-checks-and-handle-limits)
- [Choose registration and adapter options](#choose-registration-and-adapter-options)
- [Diagnose failures and respect boundaries](#diagnose-failures-and-respect-boundaries)

## The problem it solves

A profile update receives a `display_name`. The application must reject a name containing only whitespace and tell the client which field failed, while leaving the submitted text out of diagnostics. At first, a direct Rust rule may be enough. When multiple entry points choose rules from configuration, the application also needs stable rule IDs, an error if startup configuration cannot be bound, and a report it can map to its own response format.

`qubit-validator` separates those steps. `Validator<T, C>` is ordinary typed rule code. A descriptor and registry bind a selected rule once; the bound validator checks input shape on each call. `ValidationReport` attaches the caller's field path to a result. The application still chooses fields, schedules rules, decides whether to save, and creates client messages. The crate neither walks profile objects nor stores data.

The repository's [local registry example](../examples/local_registry.rs) runs with `cargo run --example local_registry --locked`. It checks the same `text.non_blank` binding with accepted and rejected names, plus named dependency binding and parameter consumption. It does not implement the profile store or collect a report; the application integration below shows those boundaries. For annotation-driven whole-object validation, see the [user registration example](../README.md#quick-start) and the [model validation guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md#core-workflow-validate-a-profile-and-keep-partial-results).

## Integrate a profile update

Add the crate to the application's `Cargo.toml`:

```toml
[dependencies]
qubit-validator = "0.1"
```

The default features are enough for this path. Define the business rule in the profile module. Its domain error stays useful to code that calls the rule directly; the configured path will map it to a stable violation code.

```rust
// src/profile/rules.rs
use std::{error::Error, fmt, sync::Arc};
use qubit_validator::{
    ArgumentReader, BindError, InputType, NamedValidationArgument,
    PreparedValidator, Validator, ValidatorDescriptor, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

#[derive(Debug)]
pub struct BlankText;
impl fmt::Display for BlankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text must not be blank")
    }
}
impl Error for BlankText {}

pub struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankText;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
    }
}

fn prepare(params: &[NamedValidationArgument<'_>])
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare),
];
pub static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
```

`finish()` rejects unexpected parameters because this rule declares none. The mapper uses a fixed code and does not copy the rejected name or the domain error's text into the violation. At application startup, register and bind the rule. Keep the resulting `BoundValidator` in the profile service rather than rebuilding it for every request:

```rust
// src/profile/startup.rs
use qubit_validator::{
    BoundValidator, InputType, RegistrationSource, ValidatorId,
    ValidatorRegistration, ValidatorRegistry,
};
use crate::profile::rules::DESCRIPTOR;

pub fn bind_display_name_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    Ok(registry.bind("text.non_blank", InputType::Text, &[])? )
}
```

`from_registrations` detects duplicate IDs and invalid descriptors; `bind` selects the text signature and prepares the rule. Handle either error before accepting profile requests. This small function returns one bound rule; an application with many rules can keep the registry and bind each selected ID at startup.

For each request, the profile service validates the borrowed name and records the result at a path chosen by the application:

```rust
// src/profile/service.rs
use qubit_validator::{
    BoundValidationContext, BoundValidator, ValidationPath,
    ValidationReport, ValidationValue,
};

fn check_display_name(
    rule: &BoundValidator,
    display_name: &str,
) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let outcome = rule.validate(
        ValidationValue::Text(display_name),
        &BoundValidationContext::new(&[]),
    )?;
    let mut report = ValidationReport::new();
    report.record_outcome(
        0,
        ValidationPath::root().with_field("display_name"),
        outcome,
    )?;
    Ok(report)
}
```

`0` is the occurrence number in this request's validation plan. The crate does not inspect the profile struct to find the field. For `"Ada"`, the report is valid; for `"   "`, it contains one `text.blank` violation at `display_name`. The profile service checks the report before using its own storage interface.

### Types used on this path

| Type | Role in the profile update |
| --- | --- |
| `Validator<str>` | Implements the business rule and its domain error. |
| `ValidatorSignature` / `ValidatorDescriptor` | Declare text input, dependencies, and preparation. |
| `ValidatorRegistry` | Finds a rule by stable ID and binds its selected signature. |
| `BoundValidator` | Reuses the prepared rule and checks input shape. |
| `ValidationOutcome` | Says valid, invalid, or skipped after execution. |
| `ValidationReport` | Records outcomes at caller-selected paths and occurrence numbers. |

## Check the result before saving

The application supplies `ProfileStore` and the HTTP handler. This service boundary makes the decision explicit:

```rust
// Continue in src/profile/service.rs; check_display_name is defined above.
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
    rule: &BoundValidator,
    user_id: &str,
    incoming_display_name: &str,
) -> Result<UpdateProfileResult, Box<dyn std::error::Error>> {
    let report = check_display_name(rule, incoming_display_name)?;
    if report.is_truncated() {
        return Err(std::io::Error::other("validation report is incomplete").into());
    }
    if !report.is_valid() {
        return Ok(UpdateProfileResult::Rejected(report));
    }
    store.save_display_name(user_id, incoming_display_name)?;
    Ok(UpdateProfileResult::Updated)
}
```

The store and response type are application code, not provided by the crate. `"Ada"` reaches `save_display_name` and returns `Updated` if that call succeeds. A blank name returns `Rejected(report)` without a store call. A store error remains an application error. The handler should check `is_truncated()` first, then iterate `report.violations()` and map each code to a controlled message. An invalid report can contain no retained violation, so do not index the first item without checking it. Keep the submitted name out of messages and logs.

| What happened | What to inspect | Application action |
| --- | --- | --- |
| `"Ada"` passes | `report.is_valid()` is true | Save through the application's store. |
| `"   "` fails the rule | `ValidationOutcome::Invalid`, then `report.violations()` contains `text.blank` | Reject the update and map the code to a field message. |
| ID, signature, or parameters are wrong | `ValidatorRegistryError` or `BindError` during startup | Fix configuration before serving requests. |
| Input shape or rule execution fails | `Err(ExecutionError)` from `validate` | Treat as a service failure, not invalid user data; diagnose through a trusted path. |
| A limited report cannot retain everything | `!recorded.complete()` or `report.is_truncated()` | Treat the report as incomplete; do not accept the update based on missing violations. |

`record_outcome` returns `RecordedOutcome`; `complete()` means that outcome fitted the report's limits, **not** that validation passed. This example uses `ValidationReport::new()` without count limits. For a limited report, check completeness and truncation as well as validity. `ValidationOutcome::Invalid` means a rule ran and rejected a value; it is not an `ExecutionError`.

The basic path ends here: bind at startup, validate in the request, and save only after a valid report. The following sections extend that same profile flow.

## Compare two profile fields

Suppose the profile form has `email` and `email_confirmation`. The confirmation rule needs the chosen email as a dependency. Declare the dependency once in the rule's signature and adapter, then bind it at startup. This self-contained module defines the rule, binds it at startup, and exposes a request-time call:

```rust
// src/profile/confirmation.rs
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, BoundValidator, DependencySpec, ExecutionError,
    InputType, NamedValidationArgument, NamedValidationDependency,
    PreparedOutcome, PreparedValidator, RegistrationSource, ValidationOutcome,
    ValidationValue, ValidatorDescriptor, ValidatorId, ValidatorRegistration,
    ValidatorRegistry, ValidatorSignature, ViolationCode, ViolationDraft,
    prepare_text_with_context,
};

static EMAIL_DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("email", InputType::Text, false),
];

fn prepare_confirmation(params: &[NamedValidationArgument<'_>])
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_with_context(EMAIL_DEPENDENCIES, |confirmation, context| {
        let email = context.text(0)?;
        Ok(if confirmation == email {
            PreparedOutcome::Valid
        } else {
            PreparedOutcome::Invalid(vec![ViolationDraft::new(
                ViolationCode::new("email.confirmation_mismatch"),
            )])
        })
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, EMAIL_DEPENDENCIES, prepare_confirmation),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

pub fn bind_confirmation_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("email.confirmation"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    Ok(registry.bind("email.confirmation", InputType::Text, &[])?)
}

pub fn check_confirmation(
    rule: &BoundValidator,
    email: &str,
    confirmation: &str,
) -> Result<ValidationOutcome, ExecutionError> {
    let dependencies = [NamedValidationDependency::new(
        "email", ValidationValue::Text(email),
    )];
    rule.validate_named(ValidationValue::Text(confirmation), &dependencies)
}
```

Call `bind_confirmation_rule()` during startup and keep its result in the profile service. During a request, call `check_confirmation(&rule, email, confirmation)` and record that outcome at `email_confirmation`. A larger application can register the descriptors together in one registry. The adapter reads the declared slot through `context.text(0)?`; a missing or malformed slot is an execution error.

Matching values yield `Valid`; different values yield `email.confirmation_mismatch`. Missing or wrongly typed required dependencies produce `ExecutionError`, which the service handles as a failure rather than a field violation. `validate_named` maps names to signature slots and allocates a temporary reorder buffer. If the caller has already established slot order, use `validate` with `BoundValidationContext::new(&values)`. Optional slots still occupy a position; use `ValidationValue::Missing` for absence. A wrong type is not absence.

## Configure a rule once at startup

If profile policy adds a maximum display-name length, read its `maximum` parameter while binding, then reuse that limit for every request. This module includes preparation and startup binding; it counts Unicode scalar values, so an application requiring grapheme or byte limits must define a different rule.

```rust
// src/profile/max_length.rs
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, BoundValidator, InputType, NamedValidationArgument,
    PreparedOutcome, PreparedValidator, RegistrationSource, ValidationArgument,
    ValidatorDescriptor, ValidatorId, ValidatorRegistration, ValidatorRegistry,
    ValidatorSignature, ViolationCode, ViolationDraft, prepare_text_with_context,
};

fn prepare_max_length(params: &[NamedValidationArgument<'_>])
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    let mut reader = ArgumentReader::new(params)?;
    let maximum = reader.required_u32("maximum")?;
    reader.finish()?;
    Ok(prepare_text_with_context(&[], move |name, _| {
        Ok(if name.chars().count() <= maximum as usize {
            PreparedOutcome::Valid
        } else {
            PreparedOutcome::Invalid(vec![ViolationDraft::new(
                ViolationCode::new("text.too_long"),
            )])
        })
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_max_length),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

pub fn bind_max_length_rule(maximum: u32)
    -> Result<BoundValidator, Box<dyn std::error::Error>>
{
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.max_length"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    let args = [NamedValidationArgument::new(
        "maximum", ValidationArgument::Unsigned(u128::from(maximum)),
    )];
    Ok(registry.bind("text.max_length", InputType::Text, &args)?)
}
```

Call `bind_max_length_rule(40)` at startup, then validate `display_name` with the bound length rule and record its outcome at the same path as the nonblank rule, using the next occurrence number. A name of at most 40 scalar values passes; a longer one produces `text.too_long`. Missing, wrongly typed, out-of-range, or unconsumed parameters fail at binding. `ArgumentReader` consumes each present parameter on the first typed read even if conversion fails; fix the configuration and bind again rather than rereading the same reader.

## Collect several checks and handle limits

The profile service decides the order of rules. Call `record_outcome` with non-decreasing occurrence numbers; multiple results may share a number. An earlier number returns `ValidationOutcomeError::OutOfOrderOccurrence` without changing the report. For example, if `email` is blank, the application can skip `email_confirmation` and point that skip to the original failure:

```rust
// non_blank_rule is the text.non_blank rule bound at startup.
use qubit_validator::{
    BoundValidationContext, ValidationOutcome, ValidationPath,
    ValidationReport, ValidationValue,
};
let mut report = ValidationReport::new();
let email_outcome = non_blank_rule.validate(
    ValidationValue::Text("   "),
    &BoundValidationContext::new(&[]),
)?;
let original = report.record_outcome(
    0,
    ValidationPath::root().with_field("email"),
    email_outcome,
)?;
let skipped = report.record_outcome(
    1,
    ValidationPath::root().with_field("email_confirmation"),
    ValidationOutcome::failed_prerequisite(original.failure_ids().to_vec())?,
)?;
assert!(original.complete() && skipped.complete());
assert_eq!(report.failure_count(), 1);
assert_eq!(report.skipped().len(), 1);
```

Only use `failed_prerequisite` when an earlier result actually retained a failure. Its `FailureId` values belong to this report and point to original violations; `report.failures()` visits each original failure once. The crate does not decide to skip a rule for you. If an optional target is missing, `ValidationOutcome::missing_optional()` records a different skip reason.

For a large request, `ValidationReport::with_limits(ValidationLimits { max_violations: Some(20), max_skipped: Some(20) })` limits retained violations and skipped entries independently. A skip's reference to an already retained failure does not consume another violation slot. Check the returned `RecordedOutcome::complete()` and `report.is_truncated()`; if application code stops checking early, call `report.mark_truncated()`. A truncated report must not be treated as proof that all remaining fields passed. These limits do not bound input length, validation work, dependency paths, prerequisite lists, or outcomes allocated before recording; the application sets those budgets separately.

## Choose registration and adapter options

Explicit `ValidatorRegistry::from_registrations` keeps rule sets local and lets startup code decide which rules are available. If separately linked crates must contribute rules process-wide, enable the optional feature:

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

Use `register_validator!` at the rule definition and `ValidatorRegistry::try_global()` at startup. `try_global` reports duplicate IDs or invalid descriptors; `global()` panics on those errors. Discovery changes how rules enter the registry, not how they bind or execute. The [repository example](../examples/local_registry.rs) includes the feature-gated path.

The simple `prepare_text_validator` and `prepare_typed_validator` adapters map a typed rule's domain error to one violation draft. For rules that read declared dependencies, use their contextual counterparts. If a rule must return several violations or distinguish an external failure from invalid input, `prepare_text_with_context` and `prepare_typed_with_context` return `PreparedOutcome` or `ExecutionError`; `prepare_text_domain_rule` and `prepare_typed_domain_rule` map contextual domain errors into safe drafts. An invalid draft list cannot be empty. `BoundValidator::try_from_prepared::<T>` binds an already prepared, dependency-free rule without a registry when its input type is exactly `T`.

## Diagnose failures and respect boundaries

| Symptom | Check and next step |
| --- | --- |
| `MissingRule` at startup | Confirm the ID is in the selected registry; with `inventory`, enable the feature and link the registering crate. |
| `UnsupportedInput` or `InputTypeMismatch` | Compare `InputType` in the signature with `ValidationValue` at the call site; text is not converted automatically. |
| `UnknownParameter` or `ParameterAlreadyConsumed` | Read each configured parameter once, call `finish()`, correct the configuration, and bind again. |
| Dependency failure | Check slot names, order, type, and required status; use `Missing` only for an optional absent value. |
| `AdapterContractViolation` | Inspect the custom prepared validator's declared input and dependencies, and ensure an invalid result has drafts. |
| `!complete()` or `is_truncated()` | Check report limits and early exits; treat the result as incomplete. |
| Field name missing from logs | `ValidationPath` formatting hides fields and map positions; render only in an authorized presentation layer. |

`ExecutionError` retains an owned cause only for explicit trusted diagnostics through `trusted_source()`; ordinary `Display`, `Debug`, and `Error::source()` do not reveal it. `ViolationParam` restricts representation, not provenance: a number from rule configuration can be safe while the same type derived from rejected input may leak data. Keep rejected and sensitive values out of codes, parameters, errors, and logs. Use stable rule IDs and violation codes when clients depend on them.

Validation runs synchronously and borrows each value for the call. Prepared validators require `Send + Sync`; the crate does not create threads or require an async runtime. The application owns object traversal, rule scheduling, request handling, storage, and message localization.

## Further reading

- [README](../README.md) · [Design and invariants](design.md) · [API reference](https://docs.rs/qubit-validator)
- [Runnable local-registry example](../examples/local_registry.rs) · [中文用户手册](user_guide.zh_CN.md)
