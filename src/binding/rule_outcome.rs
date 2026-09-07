use crate::SkipReason;
use crate::Violation;
/// Legacy prepared execution outcome.
#[derive(Debug, PartialEq, Eq)]
pub enum RuleOutcome {
    /// The value is valid.
    Valid,
    /// The value is invalid.
    Invalid(Vec<Violation>),
    /// Execution was skipped.
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
        /// Prerequisite violations.
        prerequisites: Vec<Violation>,
    },
}
