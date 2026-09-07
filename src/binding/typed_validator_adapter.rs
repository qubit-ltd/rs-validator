use std::sync::Arc;

use super::BoundValidationContext;
use super::ExecutionError;
use super::PreparedValidator;
use super::RuleOutcome;
use super::ValidationValue;
use crate::Validator;
use crate::ValidatorId;
use crate::Violation;
use crate::ViolationCode;
struct Adapter<T, V, M>(V, M, std::marker::PhantomData<fn() -> T>);
impl<T: 'static, V, M> PreparedValidator for Adapter<T, V, M>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationCode + Send + Sync + 'static,
{
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        let typed = value
            .typed::<T>()
            .ok_or_else(|| ExecutionError::new(super::ExecutionErrorKind::InputTypeMismatch))?;
        match self.0.validate(typed, &()) {
            Ok(()) => Ok(RuleOutcome::Valid),
            Err(e) => Ok(RuleOutcome::Invalid(vec![Violation::new(
                ValidatorId::new("adapter"),
                (self.1)(e),
            )])),
        }
    }
}
/// Prepares a typed validator using a domain-error mapper.
pub fn prepare_typed_validator<T: 'static, V, M>(validator: V, map_error: M) -> Arc<dyn PreparedValidator>
where
    V: Validator<T, ()> + Send + Sync + 'static,
    V::Error: Send + Sync,
    M: Fn(V::Error) -> ViolationCode + Send + Sync + 'static,
{
    Arc::new(Adapter(validator, map_error, std::marker::PhantomData))
}
