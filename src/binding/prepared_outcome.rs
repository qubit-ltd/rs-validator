use super::ViolationDraft;
#[non_exhaustive]
/// Outcome produced by a prepared validator.
pub enum PreparedOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid with draft violations.
    Invalid(Vec<ViolationDraft>),
}
