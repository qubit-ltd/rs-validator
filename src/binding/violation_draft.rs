use std::collections::BTreeMap;

use crate::ValidationPath;
use crate::ViolationCode;
use crate::ViolationParam;
/// A violation without a rule identity.
pub struct ViolationDraft {
    #[allow(dead_code)]
    code: ViolationCode,
    path: ValidationPath,
    params: BTreeMap<&'static str, ViolationParam>,
}
impl ViolationDraft {
    /// Creates a draft at the root path.
    pub fn new(code: ViolationCode) -> Self {
        Self {
            code,
            path: ValidationPath::root(),
            params: BTreeMap::new(),
        }
    }
    /// Replaces the violation path.
    pub fn with_path(mut self, path: ValidationPath) -> Self {
        self.path = path;
        self
    }
    /// Adds a structured parameter.
    pub fn with_param(mut self, name: &'static str, value: ViolationParam) -> Self {
        self.params.insert(name, value);
        self
    }
    #[allow(dead_code)]
    pub(crate) fn parts(self) -> (ViolationCode, ValidationPath, BTreeMap<&'static str, ViolationParam>) {
        (self.code, self.path, self.params)
    }
}
