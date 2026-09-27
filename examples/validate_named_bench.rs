// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::NamedValidationDependency;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorSignature;

struct Accept {
    dependencies: &'static [DependencySpec],
}

impl PreparedValidator for Accept {
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        self.dependencies
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::valid())
    }
}

static D0: &[DependencySpec] = &[];
static D1: &[DependencySpec] = &[DependencySpec::new("a", InputType::of::<u64>(), false)];
static D4: &[DependencySpec] = &[
    DependencySpec::new("a", InputType::of::<u64>(), false),
    DependencySpec::new("b", InputType::of::<u64>(), false),
    DependencySpec::new("c", InputType::of::<u64>(), false),
    DependencySpec::new("d", InputType::of::<u64>(), false),
];

fn prepare0(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Accept { dependencies: D0 }))
}

fn prepare1(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Accept { dependencies: D1 }))
}

fn prepare4(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Accept { dependencies: D4 }))
}

static S0: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, D0, prepare0)];
static S1: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, D1, prepare1)];
static S4: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, D4, prepare4)];
static V0: ValidatorDescriptor = ValidatorDescriptor::new(S0);
static V1: ValidatorDescriptor = ValidatorDescriptor::new(S1);
static V4: ValidatorDescriptor = ValidatorDescriptor::new(S4);

fn ns_per_call(iterations: u64, mut call: impl FnMut()) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        call();
    }
    start.elapsed().as_nanos() as f64 / iterations as f64
}

fn median(samples: &[f64]) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[samples.len() / 2]
}

fn run_case(
    label: &str,
    bound: &BoundValidator,
    named: &[NamedValidationDependency<'_>],
    ordered: &BoundValidationContext<'_>,
) {
    let value = ValidationValue::Text("sample");
    let mut call_named = || {
        let _ = black_box(bound.validate_named(value, named).unwrap());
    };
    let mut call_ordered = || {
        let _ = black_box(bound.validate(value, ordered).unwrap());
    };
    ns_per_call(100_000, &mut call_named);
    ns_per_call(100_000, &mut call_ordered);
    let mut named_samples = Vec::new();
    let mut ordered_samples = Vec::new();
    for round in 0..5 {
        if round % 2 == 0 {
            named_samples.push(ns_per_call(1_000_000, &mut call_named));
            ordered_samples.push(ns_per_call(1_000_000, &mut call_ordered));
        } else {
            ordered_samples.push(ns_per_call(1_000_000, &mut call_ordered));
            named_samples.push(ns_per_call(1_000_000, &mut call_named));
        }
    }
    println!(
        "{label}: named={named_samples:?} ordered={ordered_samples:?} median_named={} median_ordered={}",
        median(&named_samples),
        median(&ordered_samples),
    );
}

fn main() {
    let b0 = V0.bind(ValidatorId::new("bench.zero"), 0, &[]).unwrap();
    let b1 = V1.bind(ValidatorId::new("bench.one"), 0, &[]).unwrap();
    let b4 = V4.bind(ValidatorId::new("bench.four"), 0, &[]).unwrap();
    let number = 7_u64;
    let value = ValidationValue::Typed(&number);
    let path = ValidationPath::root().with_field("sample");

    let empty_named: [NamedValidationDependency<'_>; 0] = [];
    let empty_values: [ValidationValue<'_>; 0] = [];
    run_case("zero", &b0, &empty_named, &BoundValidationContext::new(&empty_values));

    let one_named = [NamedValidationDependency::new("a", value)];
    let one_values = [value];
    run_case("one_plain", &b1, &one_named, &BoundValidationContext::new(&one_values));
    let one_named_path = [NamedValidationDependency::new("a", value).with_path(&path)];
    let one_paths = [path.clone()];
    let one_context = BoundValidationContext::new_with_paths(&one_values, &one_paths).unwrap();
    run_case("one_path", &b1, &one_named_path, &one_context);

    let four_named = [
        NamedValidationDependency::new("a", value),
        NamedValidationDependency::new("b", value),
        NamedValidationDependency::new("c", value),
        NamedValidationDependency::new("d", value),
    ];
    let four_values = [value; 4];
    run_case(
        "four_plain",
        &b4,
        &four_named,
        &BoundValidationContext::new(&four_values),
    );
    let four_named_path = [
        NamedValidationDependency::new("a", value).with_path(&path),
        NamedValidationDependency::new("b", value).with_path(&path),
        NamedValidationDependency::new("c", value).with_path(&path),
        NamedValidationDependency::new("d", value).with_path(&path),
    ];
    let four_paths = [path.clone(), path.clone(), path.clone(), path.clone()];
    let four_context = BoundValidationContext::new_with_paths(&four_values, &four_paths).unwrap();
    run_case("four_path", &b4, &four_named_path, &four_context);
}
