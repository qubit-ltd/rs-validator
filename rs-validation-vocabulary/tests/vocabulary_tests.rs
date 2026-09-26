use qubit_validation_vocabulary::NamedValidationArgument;
use qubit_validation_vocabulary::ValidationArgument;

#[test]
fn argument_variants_preserve_values_and_borrowed_slices() {
    let bools = [true, false];
    let integers = [i128::MIN, i128::MAX];
    let unsigned = [0, u128::MAX];
    let strings = ["alpha", "beta"];
    let arguments = [
        ValidationArgument::Bool(true),
        ValidationArgument::Integer(i128::MIN),
        ValidationArgument::Unsigned(u128::MAX),
        ValidationArgument::String("value"),
        ValidationArgument::BoolList(&bools),
        ValidationArgument::IntegerList(&integers),
        ValidationArgument::UnsignedList(&unsigned),
        ValidationArgument::StringList(&strings),
    ];

    assert_eq!(arguments[0], ValidationArgument::Bool(true));
    assert_eq!(arguments[1], ValidationArgument::Integer(i128::MIN));
    assert_eq!(arguments[2], ValidationArgument::Unsigned(u128::MAX));
    assert_eq!(arguments[3], ValidationArgument::String("value"));
    assert_eq!(arguments[4], ValidationArgument::BoolList(&bools));
    assert_eq!(arguments[5], ValidationArgument::IntegerList(&integers));
    assert_eq!(arguments[6], ValidationArgument::UnsignedList(&unsigned));
    assert_eq!(arguments[7], ValidationArgument::StringList(&strings));
}

#[test]
fn argument_debug_redacts_scalar_values_and_only_reports_list_lengths() {
    let values = [
        ValidationArgument::Bool(true),
        ValidationArgument::Integer(-918273645),
        ValidationArgument::Unsigned(918273645),
        ValidationArgument::String("private-value"),
        ValidationArgument::BoolList(&[true, false]),
        ValidationArgument::IntegerList(&[-1, 2]),
        ValidationArgument::UnsignedList(&[3, 4]),
        ValidationArgument::StringList(&["first-private", "second-private"]),
    ];
    let output = format!("{values:?}");

    for secret in ["918273645", "private-value", "first-private", "second-private"] {
        assert!(!output.contains(secret), "debug output leaked {secret}");
    }
    assert!(output.contains("Bool(<redacted>)"));
    assert!(output.contains("Integer(<redacted>)"));
    assert!(output.contains("Unsigned(<redacted>)"));
    assert!(output.contains("String(<redacted>)"));
    assert!(output.contains("len: 2"));
}

#[test]
fn named_argument_borrows_name_and_value_but_redacts_both_in_debug() {
    let argument = NamedValidationArgument::new("private-name", ValidationArgument::String("private-value"));

    assert_eq!(argument.name(), "private-name");
    assert_eq!(argument.value(), ValidationArgument::String("private-value"));
    let output = format!("{argument:?}");
    assert!(!output.contains("private-name"));
    assert!(!output.contains("private-value"));
}

#[test]
#[should_panic(expected = "validator parameter name cannot be empty")]
fn named_argument_rejects_an_empty_name() {
    let _ = NamedValidationArgument::new("", ValidationArgument::Bool(false));
}
