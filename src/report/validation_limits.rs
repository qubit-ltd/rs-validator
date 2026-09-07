/// Explicit bounds applied while collecting a validation report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationLimits { /// Maximum violations. None means unlimited.
    pub max_violations: Option<usize>, /// Maximum skipped entries. None means unlimited.
    pub max_skipped: Option<usize>, }
impl Default for ValidationLimits { fn default() -> Self { Self { max_violations: None, max_skipped: None } } }
