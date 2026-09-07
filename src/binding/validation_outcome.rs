use crate::Violation;
/// Outcome produced by a bound validator.
#[non_exhaustive]
pub enum ValidationOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid.
    Invalid(Vec<Violation>),
}
