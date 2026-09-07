use crate::{SkipReason, Violation};
#[derive(Debug, PartialEq, Eq)]
/// Legacy prepared execution outcome.
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
