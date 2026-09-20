// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict, one-pass decoding of named validator parameters.

use super::BindError;
use super::BindErrorKind;
use crate::NamedValidationArgument;
use crate::ValidationArgument;

/// Reads declared arguments exactly once while rejecting duplicates, unknown
/// names, and lossy numeric conversions.
///
/// A present parameter can be read at most once. A typed read consumes the
/// parameter before decoding it, so type and range errors also complete that
/// parameter's consumption. Call [`Self::finish`] to reject parameters that
/// were never read.
///
/// # Examples
///
/// ```
/// use qubit_validator::ArgumentReader;
/// use qubit_validator::NamedValidationArgument;
/// use qubit_validator::ValidationArgument;
///
/// let arguments = [NamedValidationArgument::new(
///     "limit",
///     ValidationArgument::Unsigned(10),
/// )];
/// let mut reader = ArgumentReader::new(&arguments)?;
/// assert_eq!(reader.required_u32("limit")?, 10);
/// reader.finish()?;
/// # Ok::<(), qubit_validator::BindError>(())
/// ```
pub struct ArgumentReader<'a> {
    /// Declared parameters in caller-provided order.
    args: &'a [NamedValidationArgument<'a>],
    /// Per-parameter flags recording successful or failed read attempts.
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
            if args[..index].iter().any(|previous| previous.name() == argument.name()) {
                return Err(BindError::new(BindErrorKind::DuplicateParameter).with_parameter(argument.name()));
            }
        }
        Ok(Self {
            args,
            consumed: vec![false; args.len()],
        })
    }

    /// Reads a required unsigned 32-bit integer once.
    ///
    /// # Errors
    ///
    /// Returns `MissingParameter` when the parameter is absent,
    /// `ParameterAlreadyConsumed` when it was previously read,
    /// `ParameterTypeMismatch` when it is not an integer, or
    /// `ParameterOutOfRange` when its value cannot be represented as a `u32`.
    pub fn required_u32(&mut self, name: &str) -> Result<u32, BindError> {
        self.u32_value(name, true).map(|value| value.expect("required value"))
    }

    /// Reads an optional unsigned 32-bit integer once.
    ///
    /// # Returns
    ///
    /// Returns `Some` with the decoded value when the parameter is present,
    /// or `None` when it is absent.
    ///
    /// # Errors
    ///
    /// Returns `ParameterAlreadyConsumed` when the present parameter was
    /// previously read, `ParameterTypeMismatch` when it is not an integer, or
    /// `ParameterOutOfRange` when its value cannot be represented as a `u32`.
    pub fn optional_u32(&mut self, name: &str) -> Result<Option<u32>, BindError> {
        self.u32_value(name, false)
    }

    /// Reads a required string argument once.
    ///
    /// # Errors
    ///
    /// Returns `MissingParameter` when the parameter is absent,
    /// `ParameterAlreadyConsumed` when it was previously read, or
    /// `ParameterTypeMismatch` when it is not a string.
    pub fn required_str(&mut self, name: &str) -> Result<&'a str, BindError> {
        match self.take(name)? {
            ValidationArgument::String(value) => Ok(value),
            _ => Err(Self::type_error(name)),
        }
    }

    /// Reads an optional boolean argument once.
    ///
    /// # Returns
    ///
    /// Returns `Some` with the boolean when the parameter is present, or
    /// `None` when it is absent.
    ///
    /// # Errors
    ///
    /// Returns `ParameterAlreadyConsumed` when the present parameter was
    /// previously read or `ParameterTypeMismatch` when it is not a boolean.
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
    ///
    /// # Errors
    ///
    /// Returns `UnknownParameter` for the first argument that was never read.
    pub fn finish(&self) -> Result<(), BindError> {
        if let Some((_index, argument)) = self.args.iter().enumerate().find(|(index, _)| !self.consumed[*index]) {
            return Err(BindError::new(BindErrorKind::UnknownParameter).with_parameter(argument.name()));
        }
        Ok(())
    }

    /// Reads and converts an optional or required unsigned 32-bit parameter.
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
            ValidationArgument::Integer(_) => {
                return Err(Self::range_error(name));
            }
            _ => return Err(Self::type_error(name)),
        }
        .map_err(|_| Self::range_error(name))?;
        Ok(Some(value))
    }

    /// Takes a present parameter once, or returns `None` when it is absent.
    fn take_optional(&mut self, name: &str) -> Result<Option<ValidationArgument<'a>>, BindError> {
        let Some(index) = self.args.iter().position(|argument| argument.name() == name) else {
            return Ok(None);
        };
        if self.consumed[index] {
            return Err(BindError::new(BindErrorKind::ParameterAlreadyConsumed).with_parameter(name));
        }
        self.consumed[index] = true;
        Ok(Some(self.args[index].value()))
    }

    /// Takes a required parameter once.
    fn take(&mut self, name: &str) -> Result<ValidationArgument<'a>, BindError> {
        self.take_optional(name)?
            .ok_or_else(|| BindError::new(BindErrorKind::MissingParameter).with_parameter(name))
    }

    /// Builds a type-mismatch error without retaining the rejected value.
    #[inline]
    fn type_error(name: &str) -> BindError {
        BindError::new(BindErrorKind::ParameterTypeMismatch).with_parameter(name)
    }

    /// Builds a range error without retaining the rejected value.
    #[inline]
    fn range_error(name: &str) -> BindError {
        BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(name)
    }
}

impl std::fmt::Debug for ArgumentReader<'_> {
    /// Formats only the parameter count so argument names and values stay
    /// private.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArgumentReader")
            .field("argument_count", &self.args.len())
            .finish()
    }
}
