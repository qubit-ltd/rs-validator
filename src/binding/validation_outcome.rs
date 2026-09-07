use crate::Violation;
#[non_exhaustive]
/// Outcome produced by a bound validator.
pub enum ValidationOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid.
    Invalid(Vec<Violation>),
}
