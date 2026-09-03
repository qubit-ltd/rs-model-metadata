// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable, portable identifiers for model types.

use core::borrow::Borrow;

mod model_id_buf;
mod model_id_error;

pub use self::model_id_buf::ModelIdBuf;
pub use self::model_id_error::ModelIdError;

/// A stable identifier for a model type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::ModelId;
///
/// let id = ModelId::new("example.Account");
/// assert_eq!(id.as_str(), "example.Account");
/// assert_eq!(id.type_name(), "Account");
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(&'static str);

impl ModelId {
    /// Creates a validated model ID from a static value.
    ///
    /// # Parameters
    ///
    /// - `value`: The static model-ID string to wrap.
    ///
    /// # Returns
    ///
    /// A validated model ID borrowing `value`.
    ///
    /// # Panics
    ///
    /// Panics when `value` does not follow the stable-ID protocol. Use
    /// [`ModelId::try_new`] when callers need a recoverable error.
    #[inline(always)]
    pub const fn new(value: &'static str) -> Self {
        match Self::validate(value) {
            Ok(()) => Self(value),
            Err(_) => panic!("invalid model ID"),
        }
    }

    /// Validates and creates a model ID from a static value.
    ///
    /// # Parameters
    ///
    /// - `value`: The static model-ID string to wrap.
    ///
    /// # Returns
    ///
    /// A validated model ID borrowing `value`.
    ///
    /// # Errors
    ///
    /// Returns [`ModelIdError`] when `value` does not follow the stable-ID
    /// protocol.
    #[inline]
    pub const fn try_new(value: &'static str) -> Result<Self, ModelIdError> {
        match Self::validate(value) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Validates a model-ID string without allocating.
    ///
    /// # Parameters
    ///
    /// - `value`: The candidate model-ID string.
    ///
    /// # Errors
    ///
    /// Returns [`ModelIdError`] when `value` does not follow the stable-ID
    /// protocol.
    #[inline]
    pub const fn validate(value: &str) -> Result<(), ModelIdError> {
        validate_model_id(value)
    }

    /// Returns the complete stable model ID.
    #[must_use]
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Returns the final type-name segment of this model ID.
    #[must_use]
    pub const fn type_name(self) -> &'static str {
        let bytes = self.0.as_bytes();
        let mut index = bytes.len();
        while index > 0 {
            index -= 1;
            if bytes[index] == b'.' {
                let (_, type_name) = self.0.split_at(index + 1);
                return type_name;
            }
        }
        self.0
    }
}

impl Borrow<str> for ModelId {
    /// Borrows the stable model ID as a string slice.
    fn borrow(&self) -> &str {
        self.0
    }
}

/// Validates a stable model-ID string without allocating.
const fn validate_model_id(value: &str) -> Result<(), ModelIdError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(ModelIdError::Empty);
    }
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            if start == index || index + 1 == bytes.len() {
                return Err(ModelIdError::EmptySegment);
            }
            if let Err(error) = validate_segment(bytes, start, index) {
                return Err(error);
            }
            start = index + 1;
        }
        index += 1;
    }
    if start == bytes.len() {
        return Err(ModelIdError::EmptySegment);
    }
    validate_segment(bytes, start, bytes.len())
}

/// Validates one ASCII Java-full-class-name-style segment.
const fn validate_segment(bytes: &[u8], start: usize, end: usize) -> Result<(), ModelIdError> {
    if !is_ascii_letter(bytes[start]) {
        return Err(ModelIdError::InvalidSegment);
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !(is_ascii_letter(byte) || (byte >= b'0' && byte <= b'9') || byte == b'_') {
            return Err(ModelIdError::InvalidSegment);
        }
        index += 1;
    }
    Ok(())
}

/// Returns whether `byte` is an ASCII alphabetic character.
const fn is_ascii_letter(byte: u8) -> bool {
    (byte >= b'a' && byte <= b'z') || (byte >= b'A' && byte <= b'Z')
}
