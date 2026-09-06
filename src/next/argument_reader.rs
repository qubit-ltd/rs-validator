//! Strict, one-pass decoding of named validator parameters.

use crate::NamedValidationArgument;
use crate::ValidationArgument;

use super::BindError;
use super::BindErrorKind;

/// Reads declared arguments while rejecting duplicates, unknown names, and
/// lossy numeric conversions.
pub struct ArgumentReader<'a> {
    args: &'a [NamedValidationArgument<'a>],
    consumed: Vec<bool>,
}

impl<'a> ArgumentReader<'a> {
    /// Creates a reader and checks for duplicate parameter names.
    ///
    /// # Errors
    ///
    /// Returns `DuplicateParameter` when a name occurs more than once.
    pub fn new(args: &'a [NamedValidationArgument<'a>]) -> Result<Self, BindError> {
        for (index, argument) in args.iter().enumerate() {
            if args[..index]
                .iter()
                .any(|previous| previous.name() == argument.name())
            {
                return Err(BindError::new(BindErrorKind::DuplicateParameter)
                    .with_parameter(argument.name()));
            }
        }
        Ok(Self {
            args,
            consumed: vec![false; args.len()],
        })
    }

    /// Reads a required unsigned 32-bit integer.
    pub fn required_u32(&mut self, name: &str) -> Result<u32, BindError> {
        self.u32_value(name, true)
            .map(|value| value.expect("required value"))
    }

    /// Reads an optional unsigned 32-bit integer.
    pub fn optional_u32(&mut self, name: &str) -> Result<Option<u32>, BindError> {
        self.u32_value(name, false)
    }

    /// Reads a required string argument.
    pub fn required_str(&mut self, name: &str) -> Result<&'a str, BindError> {
        match self.take(name)? {
            ValidationArgument::String(value) => Ok(value),
            _ => Err(Self::type_error(name)),
        }
    }

    /// Reads an optional boolean argument.
    pub fn optional_bool(&mut self, name: &str) -> Result<Option<bool>, BindError> {
        let Some(value) = self.take_optional(name)? else {
            return Ok(None);
        };
        match value {
            ValidationArgument::Bool(value) => Ok(Some(value)),
            _ => Err(Self::type_error(name)),
        }
    }

    /// Rejects every argument which was not consumed by a typed reader.
    pub fn finish(&self) -> Result<(), BindError> {
        if let Some((_index, argument)) = self
            .args
            .iter()
            .enumerate()
            .find(|(index, _)| !self.consumed[*index])
        {
            return Err(
                BindError::new(BindErrorKind::UnknownParameter).with_parameter(argument.name())
            );
        }
        Ok(())
    }

    fn u32_value(&mut self, name: &str, required: bool) -> Result<Option<u32>, BindError> {
        let Some(value) = self.take_optional(name)? else {
            if required {
                return Err(BindError::new(BindErrorKind::MissingParameter).with_parameter(name));
            }
            return Ok(None);
        };
        let value = match value {
            ValidationArgument::Unsigned(value) => u32::try_from(value),
            ValidationArgument::Integer(value) if value >= 0 => u32::try_from(value as u128),
            ValidationArgument::Integer(_) => return Err(Self::range_error(name)),
            _ => return Err(Self::type_error(name)),
        }
        .map_err(|_| Self::range_error(name))?;
        Ok(Some(value))
    }

    fn take_optional(&mut self, name: &str) -> Result<Option<ValidationArgument<'a>>, BindError> {
        let Some(index) = self
            .args
            .iter()
            .position(|argument| argument.name() == name)
        else {
            return Ok(None);
        };
        self.consumed[index] = true;
        Ok(Some(self.args[index].value()))
    }

    fn take(&mut self, name: &str) -> Result<ValidationArgument<'a>, BindError> {
        self.take_optional(name)?
            .ok_or_else(|| BindError::new(BindErrorKind::MissingParameter).with_parameter(name))
    }

    fn type_error(name: &str) -> BindError {
        BindError::new(BindErrorKind::ParameterTypeMismatch).with_parameter(name)
    }

    fn range_error(name: &str) -> BindError {
        BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(name)
    }
}

impl std::fmt::Debug for ArgumentReader<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArgumentReader")
            .field("argument_count", &self.args.len())
            .finish()
    }
}
