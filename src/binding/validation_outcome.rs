use crate::SkipReason;
use crate::Violation;
/// Outcome produced by a bound validator.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationOutcome {
    /// Input is valid.
    Valid,
    /// Input is invalid.
    Invalid(Vec<Violation>),
    /// Execution was skipped.
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
        /// Prerequisite violations.
        prerequisites: Vec<Violation>,
    },
}
