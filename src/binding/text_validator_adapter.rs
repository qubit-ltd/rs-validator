use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedOutcome;
use super::PreparedValidator;
use super::ValidationValue;
use crate::Validator;
use crate::ViolationCode;
use crate::ViolationDraft;
struct Adapter<V, M>(V, M);
impl<V, M> PreparedValidator for Adapter<V, M>
where
    V: Validator<str, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationCode + Send + Sync + 'static,
{
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let text = value
            .as_text()
            .ok_or_else(|| ExecutionError::new(super::ExecutionErrorKind::InputTypeMismatch))?;
        match self.0.validate(text, &()) {
            Ok(()) => Ok(PreparedOutcome::Valid),
            Err(e) => Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new((self.1)(e))])),
        }
    }
}
/// Prepares a text validator using a domain-error mapper.
pub fn prepare_text_validator<V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<str, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationCode + Send + Sync + 'static,
{
    Arc::new(Adapter(validator, map_error))
}
