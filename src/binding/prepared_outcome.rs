use super::ViolationDraft;
/// Outcome produced by a prepared validator.
#[non_exhaustive]
pub enum PreparedOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid with draft violations.
    Invalid(Vec<ViolationDraft>),
}
